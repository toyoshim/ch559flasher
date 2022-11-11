// Copyright 2022 Takashi Toyoshima <toyoshim@gmail.com>. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be found
// in the LICENSE file.
use clap::Parser;

mod ch559;
use crate::ch559::Ch559;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    #[arg(short, long, help = "Erase program area")]
    erase: bool,
    #[arg(short, long, help = "Write FILENAME to program area")]
    write: bool,
    #[arg(short = 'c', long, help = "Compare program area with FILENAME")]
    compare: bool,

    #[arg(short = 'E', long, help = "Erase data area")]
    erase_data: bool,
    #[arg(short = 'R', long, help = "Read data area to FILENAME")]
    read_data: bool,
    #[arg(short = 'W', long, help = "Write FILENAME to data area")]
    write_data: bool,
    #[arg(short = 'C', long, help = "Compare data area with FILENAME")]
    compare_data: bool,

    #[arg(short, long, help = "Fullfill unused area with randomized values")]
    fullfill: bool,
    #[arg(short, long, help = "Random seed")]
    seed: Option<i64>,

    #[arg(help = "Filename to flash from or write into")]
    filename: Option<String>,
}

fn main() {
    let options = Options::parse();
    let mut ch559 = match Ch559::new() {
        Ok(ch559) => ch559,
        Err(e) => {
            println!("{}", e);
            std::process::exit(exitcode::USAGE);
        }
    };
    if let Some(seed) = options.seed {
        println!("random seed: {}", seed);
        ch559.set_seed(seed);
    }
    if options.erase {
        match ch559.erase() {
            Ok(()) => println!("erase: complete"),
            Err(error) => {
                println!("erase: {}", error);
                std::process::exit(exitcode::IOERR);
            }
        }
    }
    if options.write {
        if let Err(error) = ch559.erase() {
            println!("erase: {}", error);
            std::process::exit(exitcode::IOERR);
        }
        if let Some(filename) = options.filename.as_ref() {
            match ch559.write(filename, true, false, options.fullfill) {
                Ok(()) => println!("write: complete"),
                Err(error) => {
                    println!("write: {}", error);
                    std::process::exit(exitcode::IOERR);
                }
            }
        } else {
            println!("write: FILENAME should be specified");
            std::process::exit(exitcode::USAGE);
        }
    }
    if options.compare {
        if let Some(filename) = options.filename.as_ref() {
            match ch559.write(filename, false, false, options.fullfill) {
                Ok(()) => println!("compare: complete"),
                Err(error) => {
                    println!("compare: {}", error);
                    std::process::exit(exitcode::IOERR);
                }
            }
        } else {
            println!("compare: FILENAME should be specified");
            std::process::exit(exitcode::USAGE);
        }
    }
    if options.erase_data {
        match ch559.erase_data() {
            Ok(()) => println!("erase_data: complete"),
            Err(error) => {
                println!("erase_data: {}", error);
                std::process::exit(exitcode::IOERR);
            }
        }
    }
    if options.read_data {
        if let Some(filename) = options.filename.as_ref() {
            match ch559.read_data(filename) {
                Ok(()) => println!("read_data: complete"),
                Err(error) => {
                    println!("read_data: {}", error);
                    std::process::exit(exitcode::IOERR);
                }
            }
        } else {
            println!("read_data: FILENAME should be specified");
            std::process::exit(exitcode::USAGE);
        }
    }
    if options.write_data {
        if let Err(error) = ch559.erase_data() {
            println!("erase_data: {}", error);
            std::process::exit(exitcode::IOERR);
        }
        if let Some(filename) = options.filename.as_ref() {
            match ch559.write(filename, true, true, options.fullfill) {
                Ok(()) => println!("write_data: complete"),
                Err(error) => {
                    println!("write_data: {}", error);
                    std::process::exit(exitcode::IOERR);
                }
            }
        } else {
            println!("write_data: FILENAME should be specified");
            std::process::exit(exitcode::USAGE);
        }
    }
    if options.compare_data {
        if let Some(filename) = options.filename.as_ref() {
            match ch559.write(filename, false, true, options.fullfill) {
                Ok(()) => println!("compare_data: complete"),
                Err(error) => {
                    println!("compare_data: {}", error);
                    std::process::exit(exitcode::IOERR);
                }
            }
        } else {
            println!("compare_data: FILENAME should be specified");
            std::process::exit(exitcode::USAGE);
        }
    }
    std::process::exit(exitcode::OK);
}
