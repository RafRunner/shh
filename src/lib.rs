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

    /// Decode payload from an image.
    #[command(visible_alias = "d")]
    Decode {
        /// Encoded image to extract the payload from
        encoded_image: String,

        /// Optional. Output file name for the extracted payload. The original file extension is preserved.
        output: Option<String>,
    },
}

#[derive(Debug)]
pub struct Config {
    operation: Operation,
}

#[derive(Debug)]
pub enum Payload {
    File { file_name: String, bytes: Vec<u8> },
    Literal(String),
}

#[derive(Debug)]
pub enum Operation {
    Encode {
        target_image: DynamicImage,
        payload: Payload,
        output_path: PathBuf,
    },
    Decode {
        encoded_image: DynamicImage,
        output_path: Option<PathBuf>,
    },
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
                    Ok(bytes) => {
                        let file_name = Path::new(&payload)
                            .file_name()
                            .unwrap() // We can safely unwrap here because we already could read the file
                            .to_string_lossy()
                            .to_string();

                        Payload::File { file_name, bytes }
                    }
                    Err(_) => Payload::Literal(payload),
                };

                let output_path = PathBuf::from(if output.ends_with(".png") {
                    output
                } else {
                    output + ".png"
                });
                Operation::Encode {
                    target_image: input_image,
                    payload: payload_data,
                    output_path,
                }
            }
            Commands::Decode {
                encoded_image,
                output,
            } => {
                let input_image = read_image(&encoded_image)?;
                let output_path = output.map(PathBuf::from);
                Operation::Decode {
                    encoded_image: input_image,
                    output_path,
                }
            }
        };

        Ok(Self { operation })
    }

    pub fn run(self) -> Result<()> {
        match self.operation {
            Operation::Encode {
                target_image,
                payload,
                output_path,
            } => {
                let encoded = encode_image(&target_image, payload)?;
                encoded.save(&output_path)?;
                println!("Encoded image saved to '{}'", output_path.display());
            }
            Operation::Decode {
                encoded_image,
                output_path,
            } => {
                let (original_name, decoded) = decode_image(&encoded_image)?;

                let final_out_path = if let Some(output_path) = &output_path {
                    let original_ext = Path::new(&original_name)
                        .extension()
                        .map(|ext| format!(".{}", ext.to_string_lossy()))
                        .unwrap_or("".to_string());

                    format!("{}{}", output_path.display(), original_ext)
                } else {
                    format!("./{}", original_name)
                };

                fs::write(&final_out_path, decoded)?;
                println!("Decoded payload saved to '{}'", final_out_path);
                if output_path.is_some() {
                    println!("Original file name was '{}'", original_name);
                }
            }
        };

        Ok(())
    }
}

impl Payload {
    /// Extra 2 bytes for file name length, 8 bytes for payload size
    fn size(&self) -> usize {
        match self {
            Payload::File { bytes, file_name } => 2 + file_name.len() + 8 + bytes.len(),
            Payload::Literal(string) => 2 + "output.txt".len() + 8 + string.len(),
        }
    }

    fn into_bytes(self) -> Result<Vec<u8>> {
        let (bytes, file_name) = match self {
            Payload::File { bytes, file_name } => (bytes, file_name),
            Payload::Literal(string) => (string.into_bytes(), "output.txt".to_string()),
        };

        let name_len = (file_name.len() as u16).to_le_bytes();
        let bytes_len = (bytes.len() as u64).to_le_bytes();

        Ok(name_len
            .into_iter()
            .chain(file_name.into_bytes())
            .chain(bytes_len)
            .chain(bytes)
            .collect::<Vec<u8>>())
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
