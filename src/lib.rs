use crate::encode_decode::{decode_image, encode_image};
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use image::{io::Reader as ImageReader, DynamicImage};
use std::fs::{self};
use std::path::{Path, PathBuf};

pub mod encode_decode;

#[derive(Parser)]
#[command(author, version, about = "Shh: simple Rust steganography")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Encode payload in image
    #[command(visible_alias = "e")]
    Encode {
        /// Target image to encode the payload into
        target_image: String,

        /// Payload (file or string) to hide in the image
        payload: String,

        /// Output file name (always saved as PNG)
        #[arg(default_value = "encoded.png")]
        output: String,
    },

    /// Decode payload from an image. The original file extension is not preserved, you need to
    /// specify it manually
    #[command(visible_alias = "d")]
    Decode {
        /// Encoded image to extract the payload from
        encoded_image: String,

        /// Output file name for the extracted payload
        #[arg(default_value = "decoded.png")]
        output: String,
    },
}

#[derive(Debug)]
pub struct Config {
    operation: Operation,
}

#[derive(Debug)]
pub enum Payload {
    File(Vec<u8>),
    Literal(String),
}

#[derive(Debug)]
pub enum Operation {
    Encode(DynamicImage, Payload, PathBuf),
    Decode(DynamicImage, PathBuf),
}

impl Config {
    pub fn build_from_args() -> Result<Self> {
        let cli = Cli::parse();
        Self::build_from_cli(cli)
    }

    fn build_from_cli(cli: Cli) -> Result<Self> {
        let operation = match cli.command {
            Commands::Encode {
                target_image,
                payload,
                output,
            } => {
                let input_image = read_image(&target_image)?;

                let payload_data = match fs::read(&payload) {
                    Ok(bytes) => Payload::File(bytes),
                    Err(_) => Payload::Literal(payload),
                };

                let output_path = PathBuf::from(format!(
                    "{}{}",
                    output,
                    if !output.ends_with(".png") {
                        ".png"
                    } else {
                        ""
                    }
                ));
                Operation::Encode(input_image, payload_data, output_path)
            }
            Commands::Decode {
                encoded_image,
                output,
            } => {
                let input_image = read_image(&encoded_image)?;
                let output_path = PathBuf::from(output);
                Operation::Decode(input_image, output_path)
            }
        };

        Ok(Self { operation })
    }

    pub fn run(self) -> Result<()> {
        match self.operation {
            Operation::Encode(input_image, payload, output_path) => {
                let encoded = encode_image(&input_image, payload)?;
                encoded.save(output_path)?;
            }
            Operation::Decode(input_image, output_path) => {
                let decoded = decode_image(&input_image)?;
                fs::write(output_path, decoded)?;
            }
        };

        Ok(())
    }
}

impl Payload {
    fn size(&self) -> usize {
        match self {
            Payload::File(bytes) => bytes.len(),
            Payload::Literal(string) => string.as_bytes().len(),
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            Payload::File(bytes) => bytes,
            Payload::Literal(string) => string.into_bytes(),
        }
    }
}

fn read_image<P: AsRef<Path>>(path: P) -> Result<DynamicImage> {
    let path_ref = path.as_ref();

    match ImageReader::open(path.as_ref()) {
        Ok(reader) => match reader.decode() {
            Ok(image) => Ok(image),
            Err(e) => Err(anyhow!(
                "File '{}' is not an image or has the wrong format: {}",
                path_ref.display(),
                e
            )),
        },
        Err(e) => Err(anyhow!(
            "Error reading file '{}': {}",
            path_ref.display(),
            e
        )),
    }
}
