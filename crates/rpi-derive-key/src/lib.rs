#![doc = include_str!("../README.md")]

use std::io;

use rand::Rng;
use thiserror::Error;
use vcio::Vcio;

mod otp;
mod vcio;

/// Check whether the device is a Raspberry Pi.
///
/// This function simply checks whether the VCIO device `/dev/vcio` exists.
pub fn is_raspberry_pi() -> bool {
    vcio::Vcio::exists()
}

/// Randomly generate a secret key to store in OTP memory.
fn generate_secret() -> [u8; 32] {
    rand::thread_rng().gen()
}

/// Check whether the Raspberry Pi's firmware supports storing a private key.
pub fn supports_private_key() -> bool {
    // Simply check whether the firmware support reading the private key.
    Vcio::open()
        .and_then(|vcio| otp::get_private_key(&vcio))
        .is_ok()
}

/// A builder for [`Deriver`].
#[derive(Debug, Clone, Default)]
pub struct DeriverBuilder {
    /// Initialize the OTP memory.
    initialize: bool,
    /// Use the customer-programmable OTP values instead of the OTP private key.
    use_customer_otp: bool,
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
        let salt = self.salt.as_ref().map(Vec::as_slice);
        if let Ok(secret) = std::env::var("FAKE_RPI_DERIVE_KEY_SECRET") {
            // Return a `Deriver` based on the fake key.
            return Ok(Deriver::new(salt, secret.as_bytes()));
        }
        let vcio = Vcio::open()?;
        // Obtain an exclusive lock on the VCIO device.
        // let _guard = vcio.lock()?;
        let mut secret = if self.use_customer_otp {
            otp::get_customer_otp(&vcio)?
        } else {
            otp::get_private_key(&vcio)?
        };
        let is_initialized = secret != [0; 32];
        if !is_initialized {
            if self.initialize {
                secret = generate_secret();
                if self.use_customer_otp {
                    otp::set_customer_otp(&vcio, &secret)?;
                } else {
                    otp::set_private_key(&vcio, &secret)?;
                }
            } else {
                return Err(BuildError::Uninitialized);
            }
        }
        Ok(Deriver::new(salt, &secret))
    }
}

#[derive(Debug, Error)]
pub enum BuildError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Device-specific secret has not been initialized.")]
    Uninitialized,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Status {
    pub has_customer_otp: bool,
    pub has_private_key: bool,
}

pub fn status() -> Result<Status, io::Error> {
    let vcio = Vcio::open()?;
    Ok(Status {
        has_customer_otp: otp::get_customer_otp(&vcio)?.iter().any(|byte| *byte != 0),
        has_private_key: otp::get_private_key(&vcio)?.iter().any(|byte| *byte != 0),
    })
}

/// Error indicating that the length of the requested key is too long.
#[derive(Error, Debug, Clone)]
#[error("The length of the requested key is too long.")]
pub struct InvalidLength(hkdf::InvalidLength);

#[derive(Clone)]
pub struct Deriver(hkdf::Hkdf<sha3::Sha3_512>);

impl Deriver {
    fn new(salt: Option<&[u8]>, secret: &[u8]) -> Self {
        Self(hkdf::Hkdf::new(salt, secret))
    }

    /// Derive a key.
    pub fn derive_key<I: AsRef<[u8]>>(&self, info: I, key: &mut [u8]) -> Result<(), InvalidLength> {
        self.0.expand(info.as_ref(), key).map_err(InvalidLength)
    }
}

impl std::fmt::Debug for Deriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Deriver").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use rand::CryptoRng;

    #[test]
    fn rng_is_cryptographic() {
        fn check<R: CryptoRng>(_: R) {}
        check(rand::thread_rng())
    }
}
