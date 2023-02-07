#![allow(clippy::uninlined_format_args)] // Required because MSRV = 1.65.

use std::fmt::Write;

use clap::{Parser, Subcommand};
use rpi_derive_key::DeriverBuilder;

/// The command line arguments.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
struct Args {
    /// Use the customer OTP values for storing the device secret.
    #[clap(long)]
    customer_otp: bool,
    /// An optional salt to use for the HKDF algorithm.
    #[clap(long)]
    salt: Option<String>,
    /// Subcommand of the CLI.
    #[command(subcommand)]
    cmd: Command,
}

/// Subcommands of the CLI.
#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Print the status of the OTP registers and key derivation mechanism.
    Status,
    /// Irreversibly initialize the OTP registers of the Raspberry Pi.
    Init,
    /// Derive a hardware-specific key using the provided information.
    Hex {
        /// The size of the key in bytes.
        bytes: u16,
        /// Additional information used to derive the key.
        info: String,
    },
    /// Derives a UUID version 4 using the provided info material.
    Uuid { info: String },
}

fn main() {
    let args = Args::parse();

    let builder = DeriverBuilder::new()
        .with_salt(args.salt)
        .with_use_customer_otp(args.customer_otp);

    match args.cmd {
        Command::Status => {
            let status = rpi_derive_key::status().unwrap();
            println!("Has Customer OTP: {}", status.has_customer_otp);
            println!("Has Private Key: {}", status.has_private_key);
        }
        Command::Init => {
            builder.initialize(true).build().unwrap();
            let status = rpi_derive_key::status().unwrap();
            println!("Has Customer OTP: {}", status.has_customer_otp);
            println!("Has Private Key: {}", status.has_private_key);
        }
        Command::Hex { bytes, info } => {
            let deriver = builder.build().unwrap();

            let mut out = vec![0u8; bytes as usize];
            deriver.derive_key(&info, &mut out).unwrap();

            let mut formatted = String::with_capacity(2 * out.len());
            for byte in &out {
                write!(formatted, "{:02x}", byte).unwrap();
            }

            println!("{}", formatted);
        }
        Command::Uuid { info } => {
            let deriver = builder.build().unwrap();

            let mut out = [0; 16];
            deriver.derive_key(&info, &mut out).unwrap();
            let id = uuid::Builder::from_random_bytes(out).into_uuid();
            println!("{}", id);
        }
    }
}
