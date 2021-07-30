use std::fs;

mod lib;

use lib::IntCodeProgram;
use lib::Memory;
use std::vec;

fn main() {
	let input = fs::read_to_string("input.txt")
		.expect("Unable to reaad file.")
		.replace(" ", "")
		.replace("\n", "");
	let memory: vec::Vec<i64> = input.split(",").map(|cell| cell.parse().unwrap()).collect();
	let memory = Memory::new(memory);

	let program = IntCodeProgram::new(memory.clone(), vec![2]);
	let result = program.run();

	println!("{:?}", result);
}
