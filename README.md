<h1 align="center">
    RPi Derive Key
</h1>
<h4 align="center">
    A utility for deriving secure hardware-specific keys on Raspberry Pi.
<h4>

⚠️ **Caution:** This tool stores a randomly generated secret in the [_One-Time Programmable_ (OTP) memory](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#otp-register-and-bit-definitions) of the Raspberry Pi. This operation is **irreversible**. Please also **make sure you understand the provided security guarantees** before using it for anything serious.

- **Cryptographically strong** key derivation using [SHA3-512](https://en.wikipedia.org/wiki/SHA-3) and [HKDF](https://www.rfc-editor.org/rfc/rfc5869).
- **Standalone binary** with zero dependencies.

## How it works?

---

Made with ❤️ by [Silitics](https://www.silitics.com).
