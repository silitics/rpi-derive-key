//! Low-level interface to Raspberry Pi's _Video Core IO_ (VCIO) device.

use std::{io, marker::PhantomData, os::fd::RawFd, path::Path};

use nix::{
    errno::Errno,
    fcntl,
    libc::{c_char, c_int, c_short},
    sys::stat,
    unistd,
};

/// The path to the VCIO device.
const VCIO_PATH: &'static str = "/dev/vcio";

/// A handle to the VCIO device.
#[derive(Debug)]
pub(crate) struct Vcio(RawFd);

impl Vcio {
    /// Check whether the VCIO device exists.
    pub(crate) fn exists() -> bool {
        Path::new(VCIO_PATH).exists()
    }

    /// Open a handle to the VCIO device.
    pub(crate) fn open() -> Result<Self, io::Error> {
        let flags = fcntl::OFlag::O_NONBLOCK;
        let mode = stat::Mode::empty();
        fcntl::open(VCIO_PATH, flags, mode)
            .map_err(to_io_error)
            .map(Self)
    }

    /// Obtain an exclusive lock on the VCIO device.
    ///
    /// Reduces the risk of TOCTTOU race conditions when writing OTP values.
    pub(crate) fn lock(&self) -> Result<VcioLock<'_>, io::Error> {
        VcioLock::lock(self)
    }

    /// Perform an `ioctl` call to the VCIO property interface using the provided buffer.
    ///
    /// # Safety
    ///
    /// The provided `buffer` must be valid as required by the property interface.
    pub unsafe fn ioctl_property(&self, buffer: &mut [u32]) -> Result<c_int, io::Error> {
        // Violating this condition will most likely cause UB.
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

        ioctl_property(self.0, buffer.as_mut_ptr() as *mut c_char).map_err(to_io_error)
    }
}

impl Drop for Vcio {
    fn drop(&mut self) {
        // Any errors here may leave the file descriptor in a kind of
        // undefined state, hence, we simply ignore them.
        let _ = unistd::close(self.0);
    }
}

/// Converts an [`Errno`] into a proper [`io::Error`].
fn to_io_error(error: Errno) -> io::Error {
    io::Error::from_raw_os_error(error as i32)
}

/// An exclusive lock on the VCIO device.
pub(crate) struct VcioLock<'vcio> {
    fd: RawFd,
    lock: nix::libc::flock,
    _phantom_vcio: PhantomData<&'vcio Vcio>,
}

impl<'vcio> VcioLock<'vcio> {
    /// Obtain an exclusive lock for the given VCIO handle.
    fn lock(vcio: &'vcio Vcio) -> Result<Self, io::Error> {
        let lock = Self {
            fd: vcio.0,
            lock: nix::libc::flock {
                l_start: 0,
                l_len: 0, // Lock the entire file.
                l_pid: 0,
                l_type: nix::libc::F_WRLCK as c_short,
                l_whence: nix::libc::SEEK_SET as c_short,
            },
            _phantom_vcio: PhantomData,
        };
        fcntl::fcntl(lock.fd, fcntl::FcntlArg::F_SETLKW(&lock.lock)).map_err(to_io_error)?;
        Ok(lock)
    }
}

impl<'vcio> Drop for VcioLock<'vcio> {
    fn drop(&mut self) {
        self.lock.l_type = nix::libc::F_UNLCK as c_short;
        // Ignore any errors.
        let _ = fcntl::fcntl(self.fd, fcntl::FcntlArg::F_SETLK(&self.lock));
    }
}
