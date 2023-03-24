//! Utility module for securely handling and generating device secrets.

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use rand::Rng;

/// A box for securely storing secrets.
///
/// This type provides the following protection mechanisms:
///
/// - When dropped the memory is overwritten with the default value.
/// - On Linux, the memory is protected from being swapped to disk.
/// - [`Debug`] is always implemented but hides the secret.
///
/// Note that there intentionally exists no explicit method constructing [`Secret`] from a
/// value of type `T`. Of course, it is always possible to use [`DerefMut`] and then swap
/// the stored value. However, this is not recommended as it implies that at some point
/// the secret lies somewhere in unprotected memory. Instead, the secret should be
/// constructed in-place whenever possible.
///
/// We use [`Secret`] when handling the device and group secret.
pub(crate) struct Secret<T: Copy + Default>(Box<T>);

impl<T: Copy + Default> Secret<T> {
    /// Creates a new [`Secret`] using the default value of `T`.
    pub fn new() -> Self {
        Self(Box::default()).protect()
    }

    /// Protects the underlying memory from being swapped to disk (on Linux only).
    ///
    /// # Panics
    ///
    /// Panics in case the memory cannot be protected.
    fn protect(self) -> Self {
        #[cfg(target_os = "linux")]
        {
            use std::ffi::c_void;

            let result = unsafe {
                // SAFETY: Uses a valid allocation and the correct length.
                nix::libc::mlock(
                    self.0.as_ref() as *const _ as *const c_void,
                    std::mem::size_of_val(self.0.as_ref()),
                )
            };
            if result != 0 {
                panic!(
                    "Unable to `mlock` memory. {}",
                    std::io::Error::last_os_error()
                );
            }
        }
        self
    }
}

impl<T: Copy + Default> From<&T> for Secret<T> {
    fn from(value: &T) -> Self {
        let mut secret = Self::new();
        *secret = *value;
        secret
    }
}

impl<T: Copy + Default> Clone for Secret<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone()).protect()
    }
}

impl<T: Copy + Default> Default for Secret<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy + Default> Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Secret").finish_non_exhaustive()
    }
}

impl<T: Copy + Default> Deref for Secret<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: Copy + Default> DerefMut for Secret<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl<T: Copy + Default> Drop for Secret<T> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: Underlying type `T` is copy. Hence, we can just overwrite it with
            // its default value. Note that we have to use `write_volatile` here because
            // we do not want the compiler to optimize this write.
            std::ptr::write_volatile(self.0.as_mut(), T::default())
        }
    }
}

/// Type of the group secret.
pub(crate) type GroupSecret = Secret<[u8; 16]>;

/// Type of the device secret.
pub(crate) type DeviceSecret = Secret<[u8; 32]>;

/// Randomly generates a device secret using a cryptographic random number generator.
#[allow(dead_code)] // Only used on Linux.
pub(crate) fn generate_device_secret() -> DeviceSecret {
    let mut secret = DeviceSecret::new();
    rand::thread_rng().fill(secret.as_mut_slice());
    secret
}

/// Overwrites the upper 128-bits of the device secret with the provided group secret.
#[allow(dead_code)] // Only used on Linux.
pub(crate) fn set_group_secret(device_secret: &mut DeviceSecret, group_secret: &GroupSecret) {
    device_secret[..16].copy_from_slice(group_secret.as_slice());
}

/// Extracts and returns the group secret from the provided device secret.
pub(crate) fn get_group_secret(device_secret: &DeviceSecret) -> &[u8; 16] {
    device_secret[..16]
        .try_into()
        .expect("Cannot fail because the slice consists of exactly 16 bytes.")
}

#[cfg(test)]
mod tests {
    use rand::CryptoRng;

    use super::*;

    /// Checks that the used random number generator is cryptographic.
    #[test]
    fn test_rng_is_cryptographic() {
        fn check<R: CryptoRng>(_: R) {}
        check(rand::thread_rng())
    }

    /// Tests the construction and value of the default [`DeviceSecret`].
    #[test]
    fn test_default_device_secret() {
        let secret = DeviceSecret::default();
        assert_eq!(secret.deref(), &[0; 32]);
        assert_eq!(get_group_secret(&secret), &[0; 16]);
    }

    /// Tests the generation of a random secret with [`generate_device_secret`].
    #[test]
    fn test_generate_device_secret() {
        let secret = generate_device_secret();
        // Technically, the randomly generated secret could be just zeros, however, the
        // probability of this happening is absolutely negligible.
        assert_ne!(secret.deref(), &[0; 32]);
        assert_ne!(get_group_secret(&secret), &[0; 16]);
    }

    /// Tests [`set_group_secret`].
    #[test]
    fn test_set_group_secret() {
        let mut secret = generate_device_secret();
        // Technically, the randomly generated secret could be just zeros, however, the
        // probability of this happening is absolutely negligible.
        assert_ne!(get_group_secret(&secret), &[0; 16]);
        assert_ne!(secret.deref(), &[0; 32]);
        set_group_secret(&mut secret, &GroupSecret::default());
        assert_eq!(get_group_secret(&secret), &[0; 16]);
        assert_ne!(secret.deref(), &[0; 32]);
    }
}
