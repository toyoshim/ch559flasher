use clap::Parser;

mod ch559;
use crate::ch559::Ch559;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    #[arg(short, long, help = "Erase User Program Area")]
    erase: bool,
    #[arg(short = 'E', long, help = "Erase Data Area")]
    erase_data: bool,
}

fn main() {
    let options = Options::parse();
    let mut ch559 = Ch559::new();

    if !ch559.is_connected() {
        println!("CH559 Not Found");
        std::process::exit(exitcode::USAGE);
    }
    if options.erase {
        match ch559.erase() {
            Ok(()) => println!("erase: complete"),
            Err(error) => {
                println!("erase: {}", error);
                std::process::exit(exitcode::IOERR);
            }
        }
        std::process::exit(exitcode::OK);
    }
    if options.erase_data {
        match ch559.erase_data() {
            Ok(()) => println!("erase_data: complete"),
            Err(error) => {
                println!("erase: {}", error);
                std::process::exit(exitcode::IOERR);
            }
        }
        std::process::exit(exitcode::OK);
    }
}
