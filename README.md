<h1 align="center">
    RPi Derive Key 🔑
</h1>
<h4 align="center">
    A utility for deriving secure device-specific keys on Raspberry Pi.
</h4>
<p align="center">
  <a href="https://github.com/silitics/rpi-derive-key/tree/readme#%EF%B8%8F-licensing"><img alt="License: MIT OR Apache 2.0" src="https://img.shields.io/crates/l/rpi-derive-key"></a>
  <a href="https://github.com/silitics/rpi-derive-key/actions"><img alt="CI Pipeline" src="https://img.shields.io/github/actions/workflow/status/silitics/rpi-derive-key/pipeline.yml?label=pipeline"></a>
  <a href="https://github.com/silitics/rpi-derive-key/releases/latest/download/rpi-derive-key_arm64.deb"><img alt="Latest ARM64 Debian Package" src="https://img.shields.io/static/v1?label=deb arm64&message=latest&color=blue"></a>
  <a href="https://github.com/silitics/rpi-derive-key/releases/latest/download/rpi-derive-key_armhf.deb"><img alt="Latest ARMHF Debian Package" src="https://img.shields.io/static/v1?label=deb armhf&message=latest&color=blue"></a>  
  <a href="https://pypi.python.org/pypi/rpi-derive-key"><img alt="PyPi Package" src="https://img.shields.io/pypi/v/rpi-derive-key.svg?label=pypi"></a>
  <a href="https://crates.io/crates/rpi-derive-key"><img alt="RPi Derive Rust Crate" src="https://img.shields.io/crates/v/rpi-derive-key?label=crates.io"></a>  
</p>

⚠️ **Caution:** This tool is based on storing a randomly generated _device secret_ in the [_One-Time Programmable_ (OTP) memory](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#otp-register-and-bit-definitions) of the Raspberry Pi SoC. The initialization of this secret is **irreversible**. Please **make sure you understand the provided security guarantees** before using it for anything serious.

- **Cryptographically strong** key derivation using [SHA3-512](https://en.wikipedia.org/wiki/SHA-3) and [HKDF](https://www.rfc-editor.org/rfc/rfc5869).
- Statically-linked **standalone binary** with zero dependencies.
- [Rust crate](https://crates.io/crates/rpi-derive-key) and [Python package](https://pypi.org/project/rpi-derive-key/) for easy integration into your project.

#### How does it work?

Upon initialization, a randomly generated 256-bit _device secret_ is stored in the OTP memory of the Raspberry Pi SoC. Note that the OTP memory on any board can be programmed _only once_. This secret is then used as input key material for the HKDF key derivation algorithm using SHA3-512 as the hash function. This enables the derivation of multiple keys from the device secret. Each key is derived from the device secret and additional _info_ material (see HKDF). The device secret should be kept secret and `rpi-derive-key` does not provide any means of reading it directly. Using it and the info material, any key can be reconstructed. Note that the Raspberry Pi SoC does _not_ provide a hardware-protected store for the secret. Any user in the `video` group and anyone with physical access to the board can obtain the secret (unless secure boot is used). Via [secure boot](https://github.com/raspberrypi/usbboot/blob/master/secure-boot-example/README.md) it is indeed possible to prevent any unauthorized access when deploying Raspberry Pi's in untrusted environments.

If you are interested in commercial support, [please contact us](mailto:support@silitics.com?subject=[RPi%20Derive%20Key]%20Support).

## 🧑‍💻 Usage

The easiest way to properly install RPi Derive Key on a Raspberry Pi is via the official Debian packages.

On a 32-bit Raspberry Pi OS (`armhf`) run:

```
wget https://github.com/silitics/rpi-derive-key/releases/latest/download/rpi-derive-key_armhf.deb
sudo dpkg --install rpi-derive-key_armhf.deb
```

On a 64-bit Raspberry Pi OS (`arm64`) run:

```
wget https://github.com/silitics/rpi-derive-key/releases/latest/download/rpi-derive-key_arm64.deb
sudo dpkg --install rpi-derive-key_arm64.deb
```

Note that the Debian packages do not include the Python package. They merely contain the command line tool and a Systemd service for initializing the device secret during the boot process. The Python package can be installed as usual via `pip` or added as a dependency to a Python project.

For testing and debugging, the Python package is also available for MacOS, Windows, and x86_64 Linux. On these platforms, however, the device secret must be faked by setting the `FAKE_RPI_DERIVE_KEY_SECRET` environment variable (see below). The Python package is documented by its [type stub](https://github.com/silitics/rpi-derive-key/blob/main/python/rpi_derive_key.pyi).

Standalone binaries are available on the [Releases page](https://github.com/silitics/rpi-derive-key/releases).

The documentation of the Rust crate is [available on docs.rs](https://docs.rs/rpi-derive-key/).

### Initialization of the Device Secret

To derive keys, the device secret needs to be initialized first.

Using the command line tool, the device secret is irreversibly initialized with:

```
rpi-derive-key init
```

Note that the initialization may fail if the firmware does not support storing a private key in OTP memory. In this case, you can either update the firmware or use the generic customer-programmable OTP registers instead:

```
rpi-derive-key --customer-otp init
```

The switch `--customer-otp` must subsequently be provided to all commands.

The Debian package comes with a Systemd service for initializing the device secret during the boot process. This is useful to initialize devices with an image or SD card. To enable this service, run:

```
sudo systemctl enable rpi-derive-key
```

### Status Information and Checks

To print status information, run:

```
rpi-derive-key status
```

To check that the secret has been properly initialized, run:

```
rpi-derive-key check
```

This is useful when using RPi Derive Key in a script.

### Deriving a Key

To derive a key and print it in hex representation use

```
rpi-derive-key hex <BYTES> <INFO>
```

where `<BYTES>` is the key size in bytes and `<INFO>` is some arbitrary string.

For instance, to generate a 512-bit filesystem encryption key run:

```
rpi-derive-key hex 64 fs.root.encryption
```

Multiple independent keys can be generated by using different values for `<INFO>`.

To derive a [UUIDv4](https://en.wikipedia.org/wiki/Universally_unique_identifier) use

```
rpi-derive-key uuid <INFO>
```

where `<INFO>` is again some arbitrary string.

For instance, to generate a unique device id run:

```
rpi-derive-key uuid device.id
```

### Testing and Debugging

For testing and debugging purposes, you can fake a device secret by setting the `FAKE_RPI_DERIVE_KEY_SECRET` environment variable to any secret you like. Please _never use this variable in production_.

Setting this variable also bypasses initialization via `rpi-derive-key init`.

### Example Use Case

Imagine you would like to derive a unique public id and a secret identification token for each device.

To derive a unique public device id using `device.id` as `<INFO>` run:

```
rpi-derive-key uuid device.id
```

You can now safely use the resulting UUID as a public device identifier. You do not have to keep it secret because it is impossible to reconstruct other keys or the device secret from it.

In addition to the public id, you can derive a 256-bit (32 bytes) secret token with:

```
rpi-derive-key hex 32 device.secret.token
```

This secret token is supposed to be shared only with trustworthy entities, e.g., it may be sent in HTTP headers to prove the device's identity to a webserver providing device configurations:

```
wget --header "X-Secret-Token: <SECRET-TOKEN>" https://example.com/<DEVICE-ID>/config.tar.gz
```

📌 **Tip:** You should use different keys (with different info material) for different purposes (e.g., fetching updates or configurations). That way, if a key for a given purpose is compromised, all other keys remain secure.

## ⚖️ Licensing

_RPi Derive Key_ is licensed under either [MIT](https://github.com/silitics/sidex/blob/main/LICENSE-MIT) or [Apache 2.0](https://github.com/silitics/sidex/blob/main/LICENSE-APACHE) at your opinion. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache 2.0 license, shall be dual licensed as above, without any additional terms or conditions.

---

Made with ❤️ for OSS by [Silitics](https://www.silitics.com).
