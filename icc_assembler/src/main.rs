extern crate clap;
extern crate regex;

use clap::{App, Arg};
use regex::Regex;
use std::fs;
use std::io::Write;

mod lib;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg_count = args.len();
    if arg_count < 2 {
        println!("Please enter a file name to compile");
        return;
    }

    let matches = App::new("icc_assembler")
        .version("1.0.0")
        .about("Compiles an icc assembly file to an icc \"binary\"")
        .arg(
            Arg::with_name("Output")
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .help("Specifies an output file for the compiled code")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("Format")
                .short("f")
                .long("format")
                .value_name("FORMAT")
                .help("formats the resulting file with one instruction per line")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("Input")
                .help("Sets the input file to compile.")
                .required(true)
                .index(1),
        )
        .get_matches();

    let should_format = matches.is_present("Format");
    let filename = match matches.value_of("Input") {
        Some(f) => f,
        None => {
            println!("Please enter a file to compile.");
            return;
        }
    };

    let input = match fs::read_to_string(filename) {
        Ok(o) => o,
        Err(e) => {
            println!("Problem reading file {}: {}", filename, e);
            return;
        }
    };

    let re = Regex::new(r" *:( *\n*\r*)*").unwrap();
    let input = re.replace_all(input.as_str(), ":");
    let input: Vec<Vec<String>> = input
        .split("\n")
        .map(|arg| {
            arg.split(",")
                .map(|arg| arg.trim_start().trim_end().replace("\r", ""))
                .collect()
        })
        .collect();

    let mut assembler = lib::Assembler::new(input, filename.to_string());

    match assembler.compile(should_format) {
        Err(errors_found) => {
            let mut error_count = 0;
            for error in errors_found {
                if error_count > 50 {
                    break;
                }
                println!("{}", error);
                error_count += 1;
            }
            println!("Build failed with {} errors.", error_count);
        }
        Ok(output) => {
            let output_name;
            match matches.value_of("Output") {
                Some(name) => output_name = String::from(name),
                None => {
                    let res = filename[1..].find(".");
                    match res {
                        Some(i) => output_name = format!("{}.icc", String::from(&filename[0..i])),
                        None => output_name = format!("{}.icc", filename),
                    }
                }
            };

            let file = fs::File::create(&output_name);
            let mut file = match file {
                Ok(f) => f,
                Err(e) => {
                    println!("Problem creating file {}: {}", output_name, e);
                    return;
                }
            };
            file.write(output.join(",").trim_start().as_bytes())
                .unwrap();
            println!("Succesfully compiled {} to {}", filename, output_name);
        }
    }
}
