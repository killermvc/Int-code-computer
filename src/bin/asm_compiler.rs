use lib::instructions_module::{Instructions, ParameterModes};
use std::fs;
use std::io::Write;

fn main() {
	let args: Vec<String> = std::env::args().collect();
	let arg_count = args.len();
	if arg_count < 2 {
		println!("Please enter a file name to compile");
		return;
	}

	let filename = &args[1];

	let input = fs::read_to_string(filename).expect("Unable to read file");
	//Remove all blank lines
	let input = input.replace("\n\n", "\n");
	let input: Vec<Vec<&str>> = input
		.split("\n")
		.map(|arg| arg.split(" ").collect())
		.collect();

	let opcodes = Instructions::new();
	let mut output: String = String::new();
	for instruction in input {
		if instruction[0].is_empty() {
			continue;
		}
		let instr = Instructions::get_instruction_from_name(instruction[0]).unwrap();
		let arg_count = opcodes[&instr].arguments_count;
		if instruction.len() != arg_count as usize + 1 {
			panic!("Wrong number of arguments for instruction {}", instr);
		}

		let opc = instr.get_instruction_opc();
		let mut modes = Vec::new();
		let mut args: Vec<String> = Vec::new();

		for i in 0..arg_count as usize {
			let mut arg = instruction[i + 1].chars();
			let first_char = arg.next().unwrap();
			let mut arg_string: String = String::new();
			if first_char == '$' {
				modes.push(ParameterModes::Immediate);
			} else if first_char == '#' {
				modes.push(ParameterModes::Relative);
			} else if first_char.is_numeric() || first_char == '-' {
				modes.push(ParameterModes::Position);
				arg_string.push(first_char);
			} else {
				panic!("Not a numeric argument");
			}
			let arg: String = arg.collect();
			arg_string.push_str(arg.as_str());
			args.push(arg_string);
		}
		modes.reverse();

		let mut found_1 = false;
		for mode in modes {
			match mode {
				ParameterModes::Position => {
					if found_1 {
						output.push('0');
					}
				}
				ParameterModes::Immediate => {
					output.push('1');
					found_1 = true;
				}
				ParameterModes::Relative => {
					output.push('2');
					found_1 = true;
				}
				_ => panic!("Unknown mode"),
			}
		}
		if opc < 10 && found_1 {
			output.push('0');
		}
		output.push_str(opc.to_string().as_str());

		for arg in args {
			output.push(',');
			output.push_str(arg.as_str());
		}
		output.push('\n');
	}
	output.pop();

	let res = filename.find(".");
	let output_name: String;
	match res {
		Some(i) => output_name = format!("{}.icc", String::from(&filename[0..i])),
		None => output_name = format!("{}.icc", filename),
	}

	let file = fs::File::create(&output_name);
	let mut file = match file {
		Ok(f) => f,
		Err(e) => panic!("Problem creating file {}: {}", output_name, e),
	};
	file.write(output.as_bytes()).unwrap();
}
