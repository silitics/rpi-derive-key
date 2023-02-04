//! Helper functions for accessing _One Time Programmable_ (OTP) values.

use std::io;

use crate::vcio::Vcio;

/// Copy bytes from a `u32` buffer as used by the property interface to a byte slice.
fn copy_bytes(src: &[u32], mut dst: &mut [u8]) {
    // This solution seams a bit ugly. Is there a better one (without unsafe)?
    for word in src {
        dst[..4].copy_from_slice(&word.to_be_bytes());
        dst = &mut dst[4..];
    }
}

/// Encode an OTP value request for the property interface of the VCIO device.
///
/// This function only works for customer-programmable and private key OTP requests.
fn encode_request(tag: u32, value: Option<&[u8; 32]>) -> [u32; 16] {
    let mut buffer = [
        16 * 4, // Size of the buffer in bytes.
        0,      // Request code (process request).
        tag,    // The request tag.
        8 + 32, // Size of the value buffer in bytes.
        0,      // Tag request code.
        0,      // Start reading/writing at row 0.
        8,      // Read/write all 8 rows.
        0,      // 1. [u8; 4]
        0,      // 2. [u8; 4]
        0,      // 3. [u8; 4]
        0,      // 4. [u8; 4]
        0,      // 5. [u8; 4]
        0,      // 6. [u8; 4]
        0,      // 7. [u8; 4]
        0,      // 8. [u8; 4]
        0,      // End tag.
    ];
    if let Some(value) = value {
        // This solution seams a bit ugly. Is there a better one (without unsafe)?
        for (idx, word) in value.chunks(4).enumerate() {
            buffer[7 + idx] = u32::from_be_bytes(word.try_into().unwrap());
        }
    }
    buffer
}

/// Send an OTP value request to the property interface and return the result.
fn send_request(vcio: &Vcio, tag: u32, value: Option<&[u8; 32]>) -> Result<[u8; 32], io::Error> {
    let mut buffer = encode_request(tag, value);
    unsafe {
        // SAFETY: This is safe because the buffer is valid as required by the property interface.
        vcio.ioctl_property(&mut buffer)?;
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
    let mut value = [0u8; 32];
    copy_bytes(&buffer[7..15], &mut value);
    Ok(value)
}

/// Read the customer-programmable OTP values stored in rows 36 to 43.
pub(crate) fn get_customer_otp(vcio: &Vcio) -> Result<[u8; 32], io::Error> {
    send_request(vcio, 0x00030021, None)
}

/// Write the customer-programmable OTP values stored in rows 36 to 43.
///
/// This operation is irreversible.
pub(crate) fn set_customer_otp(vcio: &Vcio, value: &[u8; 32]) -> Result<[u8; 32], io::Error> {
    send_request(vcio, 0x00038021, Some(value))
}

/// Read the device-specific private key stored in OTP rows 56 to 63.
///
/// This requires a more recent firmware than [`get_customer_otp`].
pub(crate) fn get_private_key(vcio: &Vcio) -> Result<[u8; 32], io::Error> {
    send_request(vcio, 0x00030081, None)
}

/// Write the device-specific private key stored in OTP rows 56 to 63.
///
/// This operation is irreversible.
///
/// This requires a more recent firmware than [`set_customer_otp`].
pub(crate) fn set_private_key(vcio: &Vcio, value: &[u8; 32]) -> Result<[u8; 32], io::Error> {
    send_request(vcio, 0x00038081, Some(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_request_encoding() {
        // Reading of OTP values.
        assert_eq!(
            encode_request(0x00030021, None),
            [64, 0, 0x00030021, 40, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            encode_request(0x00030081, None),
            [64, 0, 0x00030081, 40, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );

        // Writing of OTP values.
        let mut value = [0; 32];
        copy_bytes(
            &[
                0xABABABAB, 0x1234ABCD, 0x00FF00FF, 0xDDAADDAA, 0xFFFFFFFF, 0xAA00BB00, 0x12345678,
                0x12121212,
            ],
            &mut value,
        );
        assert_eq!(
            encode_request(0x00038021, Some(&value)),
            [
                64, 0, 0x00038021, 40, 0, 0, 8, 0xABABABAB, 0x1234ABCD, 0x00FF00FF, 0xDDAADDAA,
                0xFFFFFFFF, 0xAA00BB00, 0x12345678, 0x12121212, 0
            ]
        );
        assert_eq!(
            encode_request(0x00038081, Some(&value)),
            [
                64, 0, 0x00038081, 40, 0, 0, 8, 0xABABABAB, 0x1234ABCD, 0x00FF00FF, 0xDDAADDAA,
                0xFFFFFFFF, 0xAA00BB00, 0x12345678, 0x12121212, 0
            ]
        );
    }
}
