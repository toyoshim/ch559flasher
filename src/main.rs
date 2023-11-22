// Copyright 2022 Takashi Toyoshima <toyoshim@gmail.com>.
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
    #[arg(short = 'w', long, help = "Write a specified file to program area")]
    write_program: Option<String>,
    #[arg(short = 'c', long, help = "Compare program area with a specified file")]
    compare_program: Option<String>,

    #[arg(short = 'E', long, help = "Erase data area")]
    erase_data: bool,
    #[arg(short = 'R', long, help = "Read data area to a specified file")]
    read_data: Option<String>,
    #[arg(short = 'W', long, help = "Write a specified file to data area")]
    write_data: Option<String>,
    #[arg(short = 'C', long, help = "Compare data area with a specified file")]
    compare_data: Option<String>,

    #[arg(short, long, help = "Fullfill unused area with randomized values")]
    fullfill: bool,
    #[arg(short, long, help = "Random seed")]
    seed: Option<u64>,

    #[arg(short = 'g', long, help = "Write BOOT_CFG[15:8] in hex (i.e. 4e)")]
    config: Option<String>,

    #[arg(short, long, help = "Boot application")]
    boot: bool,
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
    if options.erase || options.write_program.is_some() {
        match ch559.erase() {
            Ok(()) => println!("erase: complete"),
            Err(error) => {
                println!("erase: {}", error);
                std::process::exit(exitcode::IOERR);
            }
        }
    }
    if let Some(filename) = options.write_program.as_ref() {
        match ch559.write(filename, true, false, options.fullfill) {
            Ok(()) => println!("write: complete"),
            Err(error) => {
                println!("write: {}", error);
                std::process::exit(exitcode::IOERR);
            }
        }
    }
    if let Some(filename) = options.compare_program.as_ref() {
        match ch559.write(filename, false, false, options.fullfill) {
            Ok(()) => println!("compare: complete"),
            Err(error) => {
                println!("compare: {}", error);
                std::process::exit(exitcode::IOERR);
            }
        }
    }
    if options.erase_data || options.write_data.is_some() {
        match ch559.erase_data() {
            Ok(()) => println!("erase_data: complete"),
            Err(error) => {
                println!("erase_data: {}", error);
                std::process::exit(exitcode::IOERR);
            }
        }
    }
    if let Some(filename) = options.read_data.as_ref() {
        match ch559.read_data(filename) {
            Ok(()) => println!("read_data: complete"),
            Err(error) => {
                println!("read_data: {}", error);
                std::process::exit(exitcode::IOERR);
            }
        }
    }
    if let Some(filename) = options.write_data.as_ref() {
        match ch559.write(filename, true, true, options.fullfill) {
            Ok(()) => println!("write_data: complete"),
            Err(error) => {
                println!("write_data: {}", error);
                std::process::exit(exitcode::IOERR);
            }
        }
    }
    if let Some(filename) = options.compare_data.as_ref() {
        match ch559.write(filename, false, true, options.fullfill) {
            Ok(()) => println!("compare_data: complete"),
            Err(error) => {
                println!("compare_data: {}", error);
                std::process::exit(exitcode::IOERR);
            }
        }
    }
    if let Some(config) = options.config.as_ref() {
        match u8::from_str_radix(config, 16) {
            Ok(v) => match ch559.write_config(v) {
                Ok(()) => println!("write_config: complete ({:02x})", v),
                Err(error) => {
                    println!("write_config: {}", error);
                    std::process::exit(exitcode::IOERR);
                }
            },
            Err(error) => {
                println!("config: {}", error);
                std::process::exit(exitcode::USAGE);
            }
        }
    }
    if options.boot {
        match ch559.boot() {
            Ok(()) => println!("boot: complete"),
            Err(error) => {
                println!("boot: {}", error);
                std::process::exit(exitcode::IOERR);
            }
        }
    }
    std::process::exit(exitcode::OK);
}
