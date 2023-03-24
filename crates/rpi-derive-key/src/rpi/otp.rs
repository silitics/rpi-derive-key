//! Helper functions for accessing the _one-time programmable_ (OTP) device secret.

use std::io;

use crate::{
    rpi::vcio::Vcio,
    secrets::{DeviceSecret, Secret},
};

/// Copies bytes from an `u32` buffer as used by the property interface to a byte slice.
fn copy_bytes(src: &[u32], mut dst: &mut [u8]) {
    // This solution seams a bit ugly. Is there a better one (without unsafe)?
    for word in src {
        dst[..4].copy_from_slice(&word.to_be_bytes());
        dst = &mut dst[4..];
    }
}

/// Request tags for accessing OTP values.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
enum Tag {
    /// Get the customer OTP values.
    GetCustomerOtp = 0x00030021,
    /// Set the customer OTP values.
    SetCustomerOtp = 0x00038021,
    /// Get the OTP private key.
    GetPrivateKey = 0x00030081,
    /// Set the OTP private key.
    SetPrivateKey = 0x00038081,
}

/// Encodes a request for the property interface of the VCIO device.
///
/// This function only works for customer-programmable and private key OTP requests.
///
/// It uses [`Secret`] to protect the device secret.
fn encode_request(tag: Tag, value: Option<&[u8; 32]>) -> Secret<[u32; 16]> {
    let mut buffer = Secret::<[u32; 16]>::default();
    *buffer = [
        16 * 4,     // Size of the buffer in bytes.
        0,          // Request code (process request).
        tag as u32, // The request tag.
        8 + 32,     // Size of the value buffer in bytes.
        0,          // Tag request code.
        0,          // Start reading/writing at row 0.
        8,          // Read/write all 8 rows.
        0,          // 1. [u8; 4]
        0,          // 2. [u8; 4]
        0,          // 3. [u8; 4]
        0,          // 4. [u8; 4]
        0,          // 5. [u8; 4]
        0,          // 6. [u8; 4]
        0,          // 7. [u8; 4]
        0,          // 8. [u8; 4]
        0,          // End tag.
    ];
    if let Some(value) = value {
        // This solution seams a bit ugly. Is there a better one (without unsafe)?
        for (idx, word) in value.chunks(4).enumerate() {
            buffer[7 + idx] = u32::from_be_bytes(word.try_into().unwrap());
        }
    }
    buffer
}

/// Sends a request to the property interface of the VCIO device and returns the response.
fn send_request(
    vcio: &Vcio,
    tag: Tag,
    value: Option<&[u8; 32]>,
) -> Result<DeviceSecret, io::Error> {
    let mut buffer = encode_request(tag, value);
    unsafe {
        // SAFETY: The buffer is valid according to the property interface.
        vcio.ioctl_property(buffer.as_mut_slice())?;
    };
    if buffer[1] != 0x80000000 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Request to VCIO property interface unsuccessful (0x{:08X}).",
                buffer[1]
            ),
        ));
    }
    // Extract the returned device secret.
    let mut value = DeviceSecret::new();
    copy_bytes(&buffer[7..15], value.as_mut_slice());
    Ok(value)
}

/// Reads the device secret from the customer-programmable OTP registers (rows 36 to 43).
pub(crate) fn read_customer_otp(vcio: &Vcio) -> Result<DeviceSecret, io::Error> {
    send_request(vcio, Tag::GetCustomerOtp, None)
}

/// Writes the device secret to the customer-programmable OTP registers (rows 36 to 43).
///
/// ⚠️ This operation is irreversible.
pub(crate) fn write_customer_otp(vcio: &Vcio, value: &[u8; 32]) -> Result<DeviceSecret, io::Error> {
    send_request(vcio, Tag::SetCustomerOtp, Some(value))
}

/// Reads the device secret from the private key OTP registers (rows 56 to 63).
///
/// This requires a more recent firmware than [`read_customer_otp`].
pub(crate) fn read_private_key(vcio: &Vcio) -> Result<DeviceSecret, io::Error> {
    send_request(vcio, Tag::GetPrivateKey, None)
}

/// Writes the device secret to the private key OTP registers (rows 56 to 63).
///
/// ⚠️ This operation is irreversible.
///
/// This requires a more recent firmware than [`write_customer_otp`].
pub(crate) fn write_private_key(vcio: &Vcio, value: &[u8; 32]) -> Result<DeviceSecret, io::Error> {
    send_request(vcio, Tag::SetPrivateKey, Some(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the encoding of requests.
    #[test]
    pub fn test_request_encoding() {
        // Reading of OTP values.
        assert_eq!(
            encode_request(Tag::GetCustomerOtp, None).as_slice(),
            [64, 0, 0x00030021, 40, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            encode_request(Tag::GetPrivateKey, None).as_slice(),
            [64, 0, 0x00030081, 40, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );

        // Writing of OTP values.
        let mut value = [0; 32];
        #[rustfmt::skip]
        copy_bytes(
            &[
                0xABABABAB, 0x1234ABCD, 0x00FF00FF, 0xDDAADDAA,
                0xFFFFFFFF, 0xAA00BB00, 0x12345678, 0x12121212,
            ],
            &mut value,
        );
        #[rustfmt::skip]
        assert_eq!(
            encode_request(Tag::SetCustomerOtp, Some(&value)).as_slice(),
            [
                64, 0, 0x00038021, 40, 0, 0, 8,
                0xABABABAB, 0x1234ABCD, 0x00FF00FF, 0xDDAADDAA,
                0xFFFFFFFF, 0xAA00BB00, 0x12345678, 0x12121212,
                0
            ]
        );
        #[rustfmt::skip]
        assert_eq!(
            encode_request(Tag::SetPrivateKey, Some(&value)).as_slice(),
            [
                64, 0, 0x00038081, 40, 0, 0, 8,
                0xABABABAB, 0x1234ABCD, 0x00FF00FF, 0xDDAADDAA,
                0xFFFFFFFF, 0xAA00BB00, 0x12345678, 0x12121212,
                0
            ]
        );
    }
}
