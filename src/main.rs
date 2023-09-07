use anyhow::Result;
use shh::{decode_image, encode_image, Config};
use std::{env, fs, process};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });

    let out = config.output_image.clone();
    let encoded = encode_image(config)?;

    encoded.save(out)?;

    let decoded = decode_image(&encoded)?;

    fs::write("decoded.jpeg", decoded)?;

    Ok(())
}
