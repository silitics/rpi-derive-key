//! Functionality specific to the Raspberry Pi (only available on Linux).

// use std::io;

// use self::vcio::Vcio;
// use crate::{secrets::DeviceSecret, SecretLocation};

pub(crate) mod otp;
pub(crate) mod vcio;

// pub(crate) struct OtpStore {
//     vcio: Vcio,
//     location: SecretLocation,
// }

// impl OtpStore {
//     pub fn open(location: SecretLocation) -> Result<Self, io::Error> {
//         Ok(Self {
//             vcio: Vcio::open()?,
//             location,
//         })
//     }

//     pub fn read(&self) -> Result<DeviceSecret, io::Error> {
//         match self.location {
//             SecretLocation::PrivateKey => otp::read_private_key(&self.vcio),
//             SecretLocation::CustomerOtp => otp::read_customer_otp(&self.vcio),
//         }
//     }

//     pub fn write(&mut self, secret: &DeviceSecret) -> Result<DeviceSecret, io::Error> {
//         self.vcio.lock_exclusive()?;
//         if *self.read()? != *DeviceSecret::default() {
//             let _ = self.vcio.unlock();
//             return Err(io::Error::new(
//                 io::ErrorKind::Other,
//                 format!("Device secret has already been written."),
//             ));
//         };
//         let result = match self.location {
//             SecretLocation::PrivateKey => otp::write_private_key(&self.vcio, &secret),
//             SecretLocation::CustomerOtp => otp::write_customer_otp(&self.vcio,
// &secret),         };
//         let _ = self.vcio.unlock();
//         result
//     }
// }

// pub(crate) struct FakeStore {
//     secret: Option<DeviceSecret>,
// }

// impl FakeStore {
//     pub fn new(secret: Option<DeviceSecret>) -> Self {
//         Self { secret }
//     }

//     pub fn read(&self) -> Result<DeviceSecret, io::Error> {
//         Ok(self.secret.clone().unwrap_or_default())
//     }

//     pub fn write(&mut self, secret: &DeviceSecret) -> Result<DeviceSecret, io::Error> {
//         if !self.secret.is_none() {
//             return Err(io::Error::new(
//                 io::ErrorKind::Other,
//                 format!("Device secret has already been written."),
//             ));
//         }
//         self.secret = Some(secret.clone());
//         self.read()
//     }
// }
