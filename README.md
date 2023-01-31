<h1 align="center">
    RPi Derive Key 🔑
</h1>
<h4 align="center">
    A utility for deriving secure device-specific keys on Raspberry Pi.
</h4>

⚠️ **Caution:** This tool is based on storing a randomly generated _device secret_ in the [_One-Time Programmable_ (OTP) memory](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#otp-register-and-bit-definitions) of the Raspberry Pi SoC. The initialization of this secret is **irreversible**. Please **make sure you understand the provided security guarantees** before using it for anything serious.

- **Cryptographically strong** key derivation using [SHA3-512](https://en.wikipedia.org/wiki/SHA-3) and [HKDF](https://www.rfc-editor.org/rfc/rfc5869).
- Statically-linked **standalone binary** with zero dependencies.
- Rust crate and Python package for easy integration in your project.

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

## 🤔 How it Works

Upon initialization, a randomly generated 256-bit secret is stored in the OTP memory. This key is used as input key material for the HKDF key derivation algorithm.

## ⚖️ Licensing

_RPi Derive Key_ is licensed under either [MIT](https://github.com/silitics/sidex/blob/main/LICENSE-MIT) or [Apache 2.0](https://github.com/silitics/sidex/blob/main/LICENSE-APACHE) at your opinion. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache 2.0 license, shall be dual licensed as above, without any additional terms or conditions.

---

Made with ❤️ for OSS by [Silitics](https://www.silitics.com).
