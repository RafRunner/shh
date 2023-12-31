use crate::encode_decode::{decode_image, encode_image};
use image::{io::Reader as ImageReader, DynamicImage};
use std::fs::{self};
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use strum::IntoEnumIterator;
use strum_macros::{EnumDiscriminants, EnumIter};

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

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(OperationType))]
pub enum Operation {
    Encode(DynamicImage, Payload, PathBuf),
    Decode(DynamicImage, PathBuf),
    Help,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Self> {
        match Self::build_raw_error(args) {
            Ok(config) => Ok(config),
            Err(error) => Err(anyhow!("{}\nUse 'shh h' to see program usage", error)),
        }
    }

    fn build_raw_error(args: &[String]) -> Result<Self> {
        if args.is_empty() {
            return Err(anyhow!("Too few arguments!",));
        }

        let operation_type = OperationType::build(&args[0])?;
        let args = &args[1..];
        let operation = operation_type.build_operation(args)?;

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
        // Find the maximum length for aligning the descriptions
        let max_length: usize = OperationType::iter()
            .map(|operation| operation.get_help().0.len())
            .max()
            .unwrap();

        // Create the aligned help message
        let mut help_message =
            String::from("Shh: simple Rust steganography.\nUsage: shh <operation>:\n");
        for (operation, description) in OperationType::iter().map(|it| it.get_help()) {
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

    fn build_operation(&self, args: &[String]) -> Result<Operation> {
        let range = self.get_args_range();
        if !range.contains(&args.len()) {
            return Err(anyhow!(
                "Wrong number of arguments for operation {:?}! Expected a minimum of {} and maximum of {} args",
                self,
                range.start(),
                range.end()
            ));
        }

        let operation = match self {
            Self::Encode => {
                let input_image = read_image(&args[0])?;

                let payload: Payload = match fs::read(&args[1]) {
                    Ok(bytes) => Payload::File(bytes),
                    Err(_) => Payload::Literal(args[1].clone()),
                };

                Operation::Encode(input_image, payload, self.get_output_path(args.get(2)))
            }
            Self::Decode => {
                let input_image = read_image(&args[0])?;
                Operation::Decode(input_image, self.get_output_path(args.get(1)))
            }
            Self::Help => Operation::Help,
        };

        Ok(operation)
    }

    fn get_output_path(&self, provided: Option<&String>) -> PathBuf {
        let postfix = if let Self::Encode = self { ".png" } else { "" };

        PathBuf::from(
            provided
                .map(|path| path.to_owned() + postfix)
                .unwrap_or_else(|| String::from("output.png")),
        )
    }

    fn get_help(&self) -> (&'static str, &'static str) {
        match self {
            Self::Encode => ("shh e <target image> <payload (file or string)> <output file name, default is output.png, is always a png>", "encode payload in image"),
            Self::Decode => ("shh d <encoded image> <output file name, default is output.png>", "try to decode a payload from the image"),
            Self::Help => ("shh h", "show this message")
        }
    }

    fn get_args_range(&self) -> RangeInclusive<usize> {
        match self {
            Self::Encode => 2..=3,
            Self::Decode => 1..=2,
            Self::Help => 0..=usize::MAX,
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
