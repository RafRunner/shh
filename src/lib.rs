use crate::encode_decode::{encode_byte_in_bytes, u64_to_u8_array};
use image::{io::Reader as ImageReader, DynamicImage};
use image::{GenericImageView, ImageBuffer, Rgb};
use std::fs::read;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

pub mod encode_decode;

#[derive(Debug)]
pub struct Config {
    pub input_image: DynamicImage,
    pub payload: Payload,
    pub output_image: PathBuf,
}

#[derive(Debug)]
pub enum Payload {
    File(Vec<u8>),
    Literal(String),
}

pub fn encode_image(config: Config) -> Result<DynamicImage> {
    config.validate_config()?;

    let payload_size = config.payload.size() as u64;
    let payload_size_bytes = u64_to_u8_array(&payload_size);
    let payload = config.payload.into_bytes();

    let (width, height) = config.input_image.dimensions();
    let image_bytes = config.input_image.to_rgb8().into_raw();
    let chunks: Vec<[u8; 8]> = image_bytes
        .chunks_exact(8)
        .map(|chunk| chunk.try_into().expect("Impossible"))
        .collect();

    let mut output: Vec<u8> = Vec::new();
    let mut current_chunk = 0;

    for bytes in payload_size_bytes {
        let encoded = encode_byte_in_bytes(&chunks[current_chunk], &bytes);
        for byte in encoded {
            output.push(byte);
        }
        current_chunk += 1;
    }

    for bytes in payload {
        let encoded = encode_byte_in_bytes(&chunks[current_chunk], &bytes);
        for byte in encoded {
            output.push(byte);
        }
        current_chunk += 1;
    }

    for i in current_chunk..chunks.len() {
        for byte in chunks[i] {
            output.push(byte);
        }
    }

    while output.len() < image_bytes.len() {
        output.push(image_bytes[output.len() - 1]);
    }

    let image_buffer: Option<ImageBuffer<Rgb<u8>, Vec<u8>>> =
        ImageBuffer::from_raw(width, height, output);

    Ok(image_buffer
        .map(|buffer| DynamicImage::ImageRgb8(buffer))
        .unwrap())
}

pub fn decode_image(image: &DynamicImage) -> Result<Vec<u8>> {
    let image_bytes = image.to_rgb8().into_raw();
    let chunks: Vec<[u8; 8]> = image_bytes
        .chunks_exact(8)
        .map(|chunk| chunk.try_into().expect("Impossible"))
        .collect();

    let mut decoded: Vec<u8> = Vec::new();

    let mut payload_size: usize = 0;

    for i in 0..8 {
        for j in 0..8 {
            payload_size |= (((chunks[i][j] & 0b0000_0001) << j) as usize) << (i * 8);
        }
    }

    if payload_size + 8 > chunks.len() {
        return Err(anyhow!(
            "This image probably wasn't encoded. The encoded length is bigger then expected"
        ));
    }

    for i in 8..(8 + payload_size) {
        let encoded = chunks[i];
        let mut decoded_byte = 0 as u8;

        for j in 0..8 {
            decoded_byte |= (encoded[j] & 0b0000_0001) << j;
        }

        decoded.push(decoded_byte);
    }

    Ok(decoded)
}

impl Config {
    pub fn build(args: &[String]) -> Result<Self> {
        if args.len() < 2 {
            return Err(anyhow!(
                "You need to provide at least two arguments: an input image and a payload",
            ));
        }

        let input_image = read_image(&args[0])?;

        let payload: Payload = match read(PathBuf::from(&args[1])) {
            Ok(bytes) => Payload::File(bytes),
            Err(_) => Payload::Literal(args[1].clone()),
        };

        let output_image = args
            .get(2)
            .map(|path| format!("{path}.png"))
            .unwrap_or(String::from("output.png"));

        Ok(Self {
            input_image,
            payload,
            output_image: PathBuf::from(output_image),
        })
    }

    fn validate_config(&self) -> Result<()> {
        // TODO return some recoverable error in case so the user can write the image in the 2 LSBs
        if self.payload.size() * 8 > self.input_image_rgb_bytes() {
            Err(anyhow!("The payload is too big to be coded in the input image. Choose a bigger image or compress the payload."))
        } else {
            Ok(())
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
                Err(anyhow!("File '{}' is not an image.", path_ref.display()))
            }
        },
        Err(e) => {
            dbg!(e);
            panic!("Should not happen, the file exists.");
        }
    }
}
