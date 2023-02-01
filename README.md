<h1 align="center">
    RPi Derive Key 🔑
</h1>
<h4 align="center">
    A utility for deriving secure device-specific keys on Raspberry Pi.
</h4>

⚠️ **Caution:** This tool is based on storing a randomly generated _device secret_ in the [_One-Time Programmable_ (OTP) memory](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#otp-register-and-bit-definitions) of the Raspberry Pi SoC. The initialization of this secret is **irreversible**. Please **make sure you understand the provided security guarantees** before using it for anything serious.

- **Cryptographically strong** key derivation using [SHA3-512](https://en.wikipedia.org/wiki/SHA-3) and [HKDF](https://www.rfc-editor.org/rfc/rfc5869).
- Statically-linked **standalone binary** with zero dependencies.
- [Rust crate](https://crates.io/crates/rpi-derive-key) and [Python package](https://pypi.org/project/rpi-derive-key/) for easy integration into your project.

#### How does it work?

Upon initialization, a randomly generated 256-bit _device secret_ is stored in the OTP memory of the Raspberry Pi SoC. Note that the OTP memory on any board can be programmed _only once_. This secret is then used as input key material for the HKDF key derivation algorithm using SHA3-512 as the hash function. This enables the derivation of multiple keys from the device secret. Each key is derived from the derive secret and additional _info_ material (see HKDF). The device secret should be kept secret and `rpi-derive-key` does not provide any means of reading it directly. Using it and the info material, any key can be reconstructed. Note that the Raspberry Pi SoC does _not_ provide a hardware-protected store for the secret. Any user in the `video` group and anyone with physical access to the board can obtain the secret (unless secure boot is used). Via [secure boot](https://github.com/raspberrypi/usbboot/blob/master/secure-boot-example/README.md) it is indeed possible to prevent any unauthorized access when deploying Raspberry Pi's in untrusted environments.

If you are interested in commercial support, [please contact us](mailto:support@silitics.com?subject=[RPi%20Derive%20Key]%20Support).

## Usage

### Initialization of the Device Secret

Irreversibly initialize the device secret:

```
rpi-derive-key init
```

The initialization may fail if the firmware does not support storing a private key in OTP memory. You may need to update the firmware or use the generic customer-programable OTP registers instead with:

```
rpi-derive-key --customer-otp init
```

### Deriving a Key

To derive a key use

```
rpi-derive-key gen <BYTES> <INFO>
```

where `<BYTES>` is the key size in bytes and `<INFO>` is some arbitrary string.

For instance:

```
rpi-derive-key gen 32 fs.root.encryption
```

By using different values for `<INFO>` you can generate multiple independent keys.

### Examples

Derive a unique 128-bit device ID which can be turned into a [UUID](https://en.wikipedia.org/wiki/Universally_unique_identifier):

```
rpi-derive-key gen 16 device.id
```

You do not have to keep this device ID secret because it is impossible to reconstruct other keys/the device secret from it.

Generate a 256-bit secret token used to identify the device:

```
rpi-derive-key gen 32 device.secret.token
```

This secret token is supposed to be shared only with trustworthy entities, e.g., it may be sent in HTTP headers to prove the device's identity:

```
curl --header "X-Secret-Token: <SECRET-TOKEN>" https://example.com/<DEVICE-ID>/config.tar.gz
```

## 🤔 How it Works

Upon initialization, a randomly generated 256-bit secret is stored in the OTP memory. This key is used as input key material for the HKDF key derivation algorithm.

## ⚖️ Licensing

_RPi Derive Key_ is licensed under either [MIT](https://github.com/silitics/sidex/blob/main/LICENSE-MIT) or [Apache 2.0](https://github.com/silitics/sidex/blob/main/LICENSE-APACHE) at your opinion. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache 2.0 license, shall be dual licensed as above, without any additional terms or conditions.

---

Made with ❤️ for OSS by [Silitics](https://www.silitics.com).

```

```
