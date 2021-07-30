use std::fs;

mod lib;

use lib::IntCodeProgram;
use lib::Memory;
use std::vec;

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

	let program = IntCodeProgram::new(memory.clone(), program_input);
	let result = program.run();

	println!("{:?}", result);
}
