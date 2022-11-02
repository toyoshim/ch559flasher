use clap::Parser;

mod ch559;
use crate::ch559::Ch559;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Options {}

fn main() {
    let _options = Options::parse();
    let ch559 = Ch559::new();

    if !ch559.is_connected() {
        println!("CH559 Not Found");
        std::process::exit(exitcode::USAGE);
    }
}
