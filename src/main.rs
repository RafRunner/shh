use shh::Config;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    Config::build(&args)
        .and_then(|config| config.run())
        .unwrap_or_else(|err| {
            eprintln!("{err}");
            process::exit(1);
        });
}
