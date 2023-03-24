//! # RPi Derive Key 🔑
//!
//! A utility crate for deriving secure device-specific keys on Raspberry Pi.
#![allow(clippy::uninlined_format_args)] // Required because MSRV = 1.65.

use std::io;

use thiserror::Error;

use crate::secrets::GroupSecret;

pub(crate) mod secrets;

#[cfg(target_os = "linux")]
pub(crate) mod rpi;

/// The location where the device secret is stored.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SecretLocation {
    /// The device secret is stored in the private key OTP registers.
    #[default]
    PrivateKey,
    /// The device secret is stored in the customer-programmable OTP registers.
    CustomerOtp,
}

/// Checks whether the device is a Raspberry Pi.
///
/// This function simply checks whether the VCIO device `/dev/vcio` exists.
pub fn is_raspberry_pi() -> bool {
    #[cfg(target_os = "linux")]
    return rpi::vcio::Vcio::exists();
    #[cfg(not(target_os = "linux"))]
    return false;
}

/// Check whether the Raspberry Pi's firmware supports storing a private key.
pub fn supports_private_key() -> bool {
    // Simply check whether the firmware support reading the private key.
    #[cfg(target_os = "linux")]
    return rpi::vcio::Vcio::open()
        .and_then(|vcio| rpi::otp::read_private_key(&vcio))
        .is_ok();
    #[cfg(not(target_os = "linux"))]
    return true;
}

/// A builder for [`Deriver`].
#[derive(Debug, Clone, Default)]
pub struct DeriverBuilder {
    /// Initialize the OTP memory.
    initialize: bool,
    /// Use the customer-programmable OTP values instead of the OTP private key.
    use_customer_otp: bool,
    /// An optional group secret to use when initializing the device secret.
    group_secret: Option<GroupSecret>,
    /// An optional salt to use for the HKDF algorithm.
    salt: Option<Vec<u8>>,
}

impl DeriverBuilder {
    /// Creates a new [`DeriverBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the optional salt to use for the HKDF algorithm.
    #[must_use]
    pub fn with_salt<S: AsRef<[u8]>>(mut self, salt: Option<S>) -> Self {
        self.salt = salt.map(|salt| salt.as_ref().into());
        self
    }

    pub fn use_customer_otp(&self) -> bool {
        self.use_customer_otp
    }

    /// Enable the usage of the customer-programable OTP values instead of the OTP private
    /// key.
    #[must_use]
    pub fn with_use_customer_otp(mut self, enable: bool) -> Self {
        self.set_use_customer_otp(enable);
        self
    }

    pub fn set_use_customer_otp(&mut self, enable: bool) {
        self.use_customer_otp = enable;
    }

    #[must_use]
    pub fn with_group_secret(mut self, secret: &[u8; 16]) -> Self {
        self.set_group_secret(secret);
        self
    }

    pub fn set_group_secret(&mut self, secret: &[u8; 16]) {
        self.group_secret = Some(secret.into());
    }

    /// Enable the automatic initialization of the OTP memory with a randomly generated
    /// secret.
    #[must_use]
    pub fn initialize(mut self, enable: bool) -> Self {
        self.set_initialize(enable);
        self
    }

    pub fn set_initialize(&mut self, enable: bool) {
        self.initialize = enable
    }

    /// Build a [`Deriver`].
    pub fn build(self) -> Result<Deriver, BuildError> {
        let salt = self.salt.as_deref();
        if let Ok(fake_str) = std::env::var("FAKE_RPI_DERIVE_KEY_SECRET") {
            // Return a `Deriver` based on the fake key.
            eprintln!("Warning! Using fake secret.");
            let mut secret = secrets::DeviceSecret::new();
            hex::decode_to_slice(fake_str, secret.as_mut_slice()).map_err(|err| {
                BuildError::Other(format!(
                    "Unable to decode `FAKE_PRI_DERIVE_KEY_SECRET`. {:?}",
                    err
                ))
            })?;
            return Ok(Deriver::new(salt, &secret));
        }
        #[cfg(target_os = "linux")]
        {
            let mut vcio = rpi::vcio::Vcio::open()?;
            // Obtain an exclusive lock on the VCIO device. The lock is automatically
            // released when `vcio` is dropped.
            vcio.lock_exclusive()?;
            let mut secret = if self.use_customer_otp {
                rpi::otp::read_customer_otp(&vcio)?
            } else {
                rpi::otp::read_private_key(&vcio)?
            };
            let is_initialized = secret.as_slice() != [0; 32].as_slice();
            if !is_initialized {
                if self.initialize {
                    secret = secrets::generate_device_secret();
                    if self.use_customer_otp {
                        rpi::otp::write_customer_otp(&vcio, &secret)?;
                    } else {
                        rpi::otp::write_private_key(&vcio, &secret)?;
                    }
                } else {
                    return Err(BuildError::Uninitialized);
                }
            }
            Ok(Deriver::new(salt, &secret))
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(BuildError::Uninitialized)
        }
    }
}

#[derive(Debug, Error)]
pub enum BuildError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Device-specific secret has not been initialized.")]
    Uninitialized,
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Status {
    pub has_customer_otp: bool,
    pub has_private_key: bool,
}

pub fn status() -> Result<Status, io::Error> {
    #[cfg(target_os = "linux")]
    {
        let vcio = rpi::vcio::Vcio::open()?;
        let has_customer_otp = rpi::otp::read_customer_otp(&vcio)?
            .iter()
            .any(|byte| *byte != 0);
        let has_private_key = rpi::otp::read_private_key(&vcio)
            .map(|secret| secret.iter().any(|byte| *byte != 0))
            .unwrap_or_default();
        Ok(Status {
            has_customer_otp,
            has_private_key,
        })
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(Status {
            has_customer_otp: false,
            has_private_key: std::env::var("FAKE_RPI_DERIVE_KEY_SECRET").is_ok(),
        })
    }
}

/// Error indicating that the length of the requested key is too long.
#[derive(Error, Debug, Clone)]
#[error("The length of the requested key is too long.")]
pub struct InvalidLength(hkdf::InvalidLength);

/// A _deriver_ for deriving keys from a device secret using KHDF and SHA3-512.
#[derive(Clone)]
pub struct Deriver {
    /// The HKDF structure for device-specific keys.
    device_hkdf: hkdf::Hkdf<sha3::Sha3_512>,
    /// The HKDF structure for group keys.
    group_hkdf: hkdf::Hkdf<sha3::Sha3_512>,
}

impl Deriver {
    /// Creates a new [`Deriver`] with the provided salt and secrets.
    fn new_raw(salt: Option<&[u8]>, device_secret: &[u8], group_secret: &[u8]) -> Self {
        Self {
            device_hkdf: hkdf::Hkdf::new(salt, device_secret),
            group_hkdf: hkdf::Hkdf::new(salt, group_secret),
        }
    }

    /// Creates a new [`Deriver`] with the provided salt and device secret.
    fn new(salt: Option<&[u8]>, secret: &secrets::DeviceSecret) -> Self {
        Self::new_raw(salt, secret.as_slice(), secrets::get_group_secret(secret))
    }

    /// Crates a new fake [`Deriver`] with the provided salt and device secret.
    ///
    /// This is supposed to be used for testing purposes only!
    pub fn new_fake(salt: Option<&[u8]>, secret: &[u8; 32]) -> Self {
        Self::new_raw(salt, secret.as_slice(), &secret[..16])
    }

    /// Derive a device-specific key.
    pub fn derive_key<I: AsRef<[u8]>>(&self, info: I, key: &mut [u8]) -> Result<(), InvalidLength> {
        self.device_hkdf
            .expand(info.as_ref(), key)
            .map_err(InvalidLength)
    }

    /// Derive a group key (using the upper 128-bits of the device secret).
    pub fn derive_group_key<I: AsRef<[u8]>>(
        &self,
        info: I,
        key: &mut [u8],
    ) -> Result<(), InvalidLength> {
        self.group_hkdf
            .expand(info.as_ref(), key)
            .map_err(InvalidLength)
    }
}

impl std::fmt::Debug for Deriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Deriver").finish_non_exhaustive()
    }
}
