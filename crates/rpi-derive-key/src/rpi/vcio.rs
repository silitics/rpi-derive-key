//! Low-level interface to Raspberry Pi's _Video Core IO_ (VCIO) device.

use std::{io, path::Path};

use nix::{
    errno::Errno,
    fcntl,
    libc::{c_char, c_int},
    sys::stat,
    unistd,
};

/// The path to the VCIO device.
pub(crate) const VCIO_PATH: &str = "/dev/vcio";

/// A handle to the VCIO device.
#[derive(Debug)]
pub(crate) struct Vcio {
    /// The underlying file descriptor.
    fd: c_int,
    /// Indicates whether the VCIO device has been locked.
    locked: bool,
}

impl Vcio {
    /// Checks whether the VCIO device exists.
    pub(crate) fn exists() -> bool {
        Path::new(VCIO_PATH).exists()
    }

    /// Opens a handle to the VCIO device.
    pub(crate) fn open() -> Result<Self, io::Error> {
        let flags = fcntl::OFlag::O_NONBLOCK;
        let mode = stat::Mode::empty();
        fcntl::open(VCIO_PATH, flags, mode)
            .map_err(to_io_error)
            .map(|fd| Self { fd, locked: false })
    }

    /// Generates an error when the VCIO device has already been locked.
    fn error_when_locked(&self) -> Result<(), io::Error> {
        if self.locked {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "VCIO device is already locked.",
            ))
        } else {
            Ok(())
        }
    }

    /// Obtains a shared lock on the VCIO device.
    ///
    /// Reduces the risk of race conditions when accessing OTP values.
    ///
    /// This method does not block but may return [`io::ErrorKind::WouldBlock`].
    ///
    /// Note that the lock is automatically released when [`Vcio`] is dropped.
    ///
    /// # Errors
    ///
    /// Produces an error when the VCIO device is already locked (using this handle) or
    /// the underlying call to `flock` fails.
    #[allow(dead_code)] // Currently unused but may be useful in the future.
    pub(crate) fn lock_shared(&mut self) -> Result<(), io::Error> {
        self.error_when_locked()?;

        let result = unsafe { nix::libc::flock(self.fd, nix::libc::LOCK_SH | nix::libc::LOCK_NB) };
        if result != 0 {
            Err(io::Error::last_os_error())
        } else {
            self.locked = true;
            Ok(())
        }
    }

    /// Obtains an exclusive lock on the VCIO device.
    ///
    /// Reduces the risk of race conditions when accessing OTP values.
    ///
    /// This method does not block but may return [`io::ErrorKind::WouldBlock`].
    ///
    /// Note that the lock is automatically released when [`Vcio`] is dropped.
    ///
    /// # Errors
    ///
    /// Produces an error when the VCIO device is already locked (using this handle) or
    /// the underlying call to `flock` fails.
    pub(crate) fn lock_exclusive(&mut self) -> Result<(), io::Error> {
        self.error_when_locked()?;

        let result = unsafe { nix::libc::flock(self.fd, nix::libc::LOCK_EX | nix::libc::LOCK_NB) };
        if result != 0 {
            Err(io::Error::last_os_error())
        } else {
            self.locked = true;
            Ok(())
        }
    }

    /// Releases the previously obtained lock on the VCIO device.
    ///
    /// # Errors
    ///
    /// Produces an error when the VCIO device has not been locked (using this handle) or
    /// the underlying call to `flock` fails.
    #[allow(dead_code)] // Currently unused but may be useful in the future.
    pub(crate) fn unlock(&mut self) -> Result<(), io::Error> {
        if !self.locked {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "VCIO device is not locked.",
            ));
        }

        let result = unsafe { nix::libc::flock(self.fd, nix::libc::LOCK_UN) };
        if result != 0 {
            Err(io::Error::last_os_error())
        } else {
            self.locked = false;
            Ok(())
        }
    }

    /// Performs an `ioctl` call to the VCIO property interface using the provided buffer.
    ///
    /// # Safety
    ///
    /// The provided `buffer` must be valid as required by the property interface.
    pub unsafe fn ioctl_property(&self, buffer: &mut [u32]) -> Result<c_int, io::Error> {
        // Violating this safety precondition will most likely cause UB.
        assert!(
            buffer[0] <= (buffer.len() * 4) as u32,
            "Invalid buffer size. Buffer is smaller than indicated."
        );

        /// The `ioctl` identifier of the property interface.
        const IOCTL_IDENTIFIER: u8 = 100;
        /// The `ioctl` sequence number of the property interface.
        const IOCTL_SEQ_PROPERTY: u8 = 0;

        // We have to cast to `c_int` to make this work on 32-bit and 64-bit.
        const IOCTL_REQUEST_CODE: c_int = nix::request_code_readwrite!(
            IOCTL_IDENTIFIER,
            IOCTL_SEQ_PROPERTY,
            std::mem::size_of::<*mut c_char>()
        ) as c_int;

        // We have to use `ioctl_readwrite_bad` here because the code is computed with
        // `*mut c_char` but the actual type needs to be `c_char`.
        nix::ioctl_readwrite_bad! {
            /// Raw `ioctl` call.
            ioctl_property,
            IOCTL_REQUEST_CODE,
            c_char
        };

        ioctl_property(self.fd, buffer.as_mut_ptr() as *mut c_char).map_err(to_io_error)
    }
}

impl Drop for Vcio {
    fn drop(&mut self) {
        // Any errors here may leave the file descriptor in a kind of
        // undefined state, hence, we simply ignore them.
        let _ = unistd::close(self.fd);
    }
}

/// Converts an [`Errno`] into a proper [`io::Error`].
fn to_io_error(error: Errno) -> io::Error {
    io::Error::from_raw_os_error(error as i32)
}
