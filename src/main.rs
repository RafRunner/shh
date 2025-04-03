use shh::Config;
use std::process;

fn main() {
    Config::build_from_args()
        .and_then(|config| config.run())
        .unwrap_or_else(|err| {
            eprintln!("{err}");
            process::exit(1);
        });
}
