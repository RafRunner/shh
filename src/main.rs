use anyhow::Result;
use shh::Config;
use std::{env, process};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });

    config.run()
}
