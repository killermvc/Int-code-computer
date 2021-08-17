use std::fs;

mod intcode;
pub mod lib;

use crate::lib::Memory;
use intcode::IntCodeProgram;
use std::vec;

use std::io;

fn main() {
	let args: Vec<String> = std::env::args().collect();
	let arg_count = args.len();
	if arg_count < 2 {
		println!("Please enter a file name to execute");
		return;
	}
	let mut program_input = Vec::new();
	if arg_count > 2 {
		for i in 2..arg_count {
			let temp = match args[i].parse() {
				Ok(val) => val,
				Err(e) => panic!("Error when parsing program input: {}", e),
			};
			program_input.push(temp);
		}
	}
	let input = fs::read_to_string(&args[1])
		.expect("Unable to reaad file.")
		.replace(" ", "")
		.replace(",\n", ",")
		.replace("\n", ",");
	let memory: vec::Vec<i64> = input.split(",").map(|cell| cell.parse().unwrap()).collect();
	let memory = Memory::new(memory);

	let program = IntCodeProgram::new(
		memory.clone(),
		program_input,
		|val| println!("Program Output: {}", val),
		|| {
			println!("Please enter an integer value");
			let stdin = io::stdin();
			let mut input = String::new();
			stdin.read_line(&mut input).unwrap();
			input
				.replace("\n", "")
				.replace("\r", "")
				.parse::<i64>()
				.unwrap()
		},
	);
	let result = program.run();
	println!("{:?}", result);
}
