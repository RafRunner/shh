use crate::encode_decode::{decode_image, encode_image};
use image::{io::Reader as ImageReader, DynamicImage};
use std::fs::{self};
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

pub mod encode_decode;

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
    Help,
}

#[derive(Debug)]
pub enum OperationType {
    Encode,
    Decode,
    Help,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Self> {
        if args.is_empty() {
            return Err(anyhow!(
                "Too few arguments!\n{}",
                Config::get_help_message()
            ));
        }

        let operation_type = OperationType::build(&args[0])?;
        let args = &args[1..];

        let range = operation_type.get_args_range();
        if !range.contains(&args.len()) {
            return Err(anyhow!(
                "Wrong number of arguments for operation {:?}! Expected a minimum of {} and maximum of {} args",
                operation_type,
                range.start(),
                range.end()
            ));
        }

        let operation = match operation_type {
            OperationType::Encode => {
                let input_image = read_image(&args[0])?;

                let payload: Payload = match fs::read(&args[1]) {
                    Ok(bytes) => Payload::File(bytes),
                    Err(_) => Payload::Literal(args[1].clone()),
                };

                Operation::Encode(
                    input_image,
                    payload,
                    operation_type.get_output_path(args.get(2)),
                )
            }
            OperationType::Decode => {
                let input_image = read_image(&args[0])?;
                Operation::Decode(
                    input_image,
                    operation_type.get_output_path(args.get(1)),
                )
            }
            OperationType::Help => Operation::Help,
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
            Operation::Help => println!("{}", Self::get_help_message()),
        };

        Ok(())
    }

    fn get_help_message() -> String {
        // Define the operations
        let operations = vec![
            OperationType::Encode,
            OperationType::Decode,
            OperationType::Help,
        ];

        // Find the maximum length for aligning the descriptions
        let max_length: usize = operations
            .iter()
            .map(|operation| operation.get_help().0.len())
            .max()
            .unwrap();

        // Create the aligned help message
        let mut help_message = String::from("usage: shh <operation: encode or decode>:\n");
        for (operation, description) in operations.iter().map(|it| it.get_help()) {
            let padding = " ".repeat(max_length - operation.len() + 1);
            help_message.push_str(&format!("\t{}{}({})\n", operation, padding, description));
        }

        help_message
    }
}

impl OperationType {
    fn build(command_name: &str) -> Result<Self> {
        match command_name {
            "e" | "encode" => Ok(Self::Encode),
            "d" | "decode" => Ok(Self::Decode),
            "h" | "help" => Ok(Self::Help),
            _ => Err(anyhow!("Operation {} does not exist!", command_name)),
        }
    }

    fn get_output_path(&self, provided: Option<&String>) -> PathBuf {
        let postfix = if let Self::Encode = self {
            ".png"
        } else {
            ""
        };

        PathBuf::from(
            provided
                .map(|path| path.to_owned() + postfix)
                .unwrap_or_else(|| String::from("output.png")),
        )
    }

    fn get_help(&self) -> (&'static str, &'static str) {
        match self {
            OperationType::Encode => ("shh e <target image> <payload (file or string)> <output file name, default is output.png, is always a png>", "encode payload in image"),
            OperationType::Decode => ("shh d <encoded image> <output file name, default is output.png>", "try to decode a payload from the image"),
            OperationType::Help => ("shh h", "show this message")
        }
    }

    fn get_args_range(&self) -> RangeInclusive<usize> {
        match self {
            OperationType::Encode => 2..=3,
            OperationType::Decode => 1..=2,
            OperationType::Help => 0..=usize::MAX,
        }
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
