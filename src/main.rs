// Copyright 2022 Takashi Toyoshima <toyoshim@gmail.com>. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be found
// in the LICENSE file.
use clap::Parser;

mod ch559;
use crate::ch559::Ch559;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    #[arg(short, long, help = "Erase user program area")]
    erase: bool,

    #[arg(short = 'E', long, help = "Erase data area")]
    erase_data: bool,
    #[arg(short = 'R', long, help = "Read data area to FILENAME")]
    read_data: bool,

    #[arg(help = "Filename to flash or write")]
    filename: Option<String>,
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
                println!("erase_data: {}", error);
                std::process::exit(exitcode::IOERR);
            }
        }
        std::process::exit(exitcode::OK);
    }
    if options.read_data {
        if let Some(filename) = options.filename {
            match ch559.read_data(filename) {
                Ok(()) => println!("read_data: complete"),
                Err(error) => {
                    println!("read_data: {}", error);
                    std::process::exit(exitcode::IOERR);
                }
            }
            std::process::exit(exitcode::OK);
        } else {
            println!("read_data: FILENAME should be specified");
            std::process::exit(exitcode::USAGE);
        }
    }
}
