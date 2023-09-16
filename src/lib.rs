use crate::encode_decode::{decode_image, encode_image};
use image::{io::Reader as ImageReader, DynamicImage};
use std::fs::{self};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

pub mod encode_decode;

#[derive(Debug)]
pub struct Config {
    input_image: DynamicImage,
    operaion: OperationType,
    output_path: PathBuf,
}

#[derive(Debug)]
pub enum Payload {
    File(Vec<u8>),
    Literal(String),
}

#[derive(Debug)]
pub enum OperationType {
    Encode(Payload),
    Decode,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Self> {
        if args.is_empty() {
            return Err(anyhow!(
                "Too few arguments!\n{}",
                Config::get_help_message()
            ));
        }

        let len = args.len();

        match &(*args[0]) {
            "e" | "encode" if (3..=4).contains(&len) => {
                let input_image = read_image(&args[1])?;
                if len < 3 {
                    return Err(anyhow!("please provide a payload (file or string)",));
                }

                let payload: Payload = match fs::read(&args[2]) {
                    Ok(bytes) => Payload::File(bytes),
                    Err(_) => Payload::Literal(args[2].clone()),
                };

                Ok(Self {
                    input_image,
                    operaion: OperationType::Encode(payload),
                    output_path: Self::get_output_path(args.get(3), ".png"),
                })
            }
            "d" | "decode" if (2..=3).contains(&len) => {
                let input_image = read_image(&args[1])?;
                Ok(Self {
                    input_image,
                    operaion: OperationType::Decode,
                    output_path: Self::get_output_path(args.get(2), ""),
                })
            }
            "h" | "help" => Err(anyhow!(Config::get_help_message())),
            _ => Err(anyhow!(
                "Wrong usage. Help:\n{}",
                Config::get_help_message()
            )),
        }
    }

    pub fn run(self) -> Result<()> {
        match self.operaion {
            OperationType::Encode(payload) => {
                let encoded = encode_image(&self.input_image, payload)?;
                encoded.save(self.output_path)?;
            }
            OperationType::Decode => {
                let decoded = decode_image(&self.input_image)?;
                fs::write(self.output_path, decoded)?;
            }
        };

        Ok(())
    }

    fn get_output_path(provided: Option<&String>, postfix: &str) -> PathBuf {
        PathBuf::from(
            provided
                .map(|path| path.to_owned() + postfix)
                .unwrap_or_else(|| String::from("output.png")),
        )
    }

    fn get_help_message() -> String {
        // Define the operations and their descriptions
        let operations = vec![
            ("shh e <target image> <payload (file or string)> <output file name, default is output.png, is always a png>", "encode payload in image"),
            ("shh d <encoded image> <output file name, default is output.png>", "try to decode a payload from the image"),
            ("shh h", "show this message"),
        ];

        // Find the maximum length for aligning the descriptions
        let max_length: usize = operations
            .iter()
            .map(|(operation, _)| operation.len())
            .max()
            .unwrap();

        // Create the aligned help message
        let mut help_message = String::from("usage: shh <operation: encode or decode>:\n");
        for (operation, description) in operations {
            let padding = " ".repeat(max_length - operation.len() + 1);
            help_message.push_str(&format!("\t{}{}({})\n", operation, padding, description));
        }

        help_message
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
                e.to_string()
            )),
        },
        Err(e) => Err(anyhow!(
            "Error reading file '{}': {}",
            path_ref.display(),
            e.to_string()
        )),
    }
}
