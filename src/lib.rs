use crate::encode_decode::{decode_image, encode_image};
use image::GenericImageView;
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
        if args.len() < 2 {
            return Err(anyhow!(
                "usage: shh <operaion: encode or decode>:
                    shh e <target image> <payload (file or string)> <output file (optional)>
                    shh d <encoded image> <output file (optional)>",
            ));
        }

        let input_image = read_image(&args[1])?;

        match &(*args[0]) {
            "e" | "encode" => {
                if args.len() < 3 {
                    return Err(anyhow!("please provide a payload (file or string)",));
                }

                let payload: Payload = match fs::read(&args[2]) {
                    Ok(bytes) => Payload::File(bytes),
                    Err(_) => Payload::Literal(args[1].clone()),
                };

                Ok(Self {
                    input_image,
                    operaion: OperationType::Encode(payload),
                    output_path: Self::get_output_path(args.get(3), ".png"),
                })
            }
            "d" | "decode" => Ok(Self {
                input_image,
                operaion: OperationType::Decode,
                output_path: Self::get_output_path(args.get(2), ""),
            }),
            _ => Err(anyhow!(
                "{} is not a valid operation. use d|decode or e|encode",
                args[0]
            )),
        }
    }

    pub fn run(self) -> Result<()> {
        self.validate_config()?;

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
                .unwrap_or(String::from("output.png")),
        )
    }

    fn validate_config(&self) -> Result<()> {
        match &self.operaion {
            OperationType::Decode => Ok(()),
            OperationType::Encode(payload) => {
                // TODO return some recoverable error in case so the user can write the image in the 2 LSBs
                if payload.size() * 8 + 8 > self.input_image_rgb_bytes() {
                    Err(anyhow!("The payload is too big to be coded in the input image. Choose a bigger image (in resolution) or compress the payload."))
                } else {
                    Ok(())
                }
            }
        }
    }

    fn input_image_rgb_bytes(&self) -> usize {
        let (width, height) = self.input_image.dimensions();
        width as usize * height as usize * 3
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
    if !path_ref.exists() {
        return Err(anyhow!("'{}' does not exist!", path_ref.display()));
    }

    match ImageReader::open(path_ref) {
        Ok(reader) => match reader.decode() {
            Ok(image) => Ok(image),
            Err(e) => {
                dbg!(e);
                Err(anyhow!(
                    "File '{}' is not an image or has the wrong format.",
                    path_ref.display()
                ))
            }
        },
        Err(e) => {
            dbg!(e);
            panic!("Should not happen, the file exists.");
        }
    }
}
