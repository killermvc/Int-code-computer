extern crate regex;
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

    let filename = &args[1];

    let input = fs::read_to_string(filename).expect("Unable to read file");

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

    match assembler.compile() {
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
            let res = filename[1..].find(".");
            let output_name: String;
            match res {
                Some(i) => output_name = format!("{}.icc", String::from(&filename[0..i])),
                None => output_name = format!("{}.icc", filename),
            }

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
