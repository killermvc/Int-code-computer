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
	let input: Vec<Vec<String>> = input
		.split("\n")
		.map(|arg| arg.split(" ").map(|arg| arg.replace("\r", "")).collect())
		.collect();

	let mut output: Vec<String> = Vec::new();
	let opcodes = Instructions::new();
	let mut line = 0;
	for mut instruction in input {
		line += 1;
		if instruction[0].is_empty() || instruction[0].find("\r").unwrap_or(1) == 0 {
			continue;
		}
		let instr = Instructions::get_instruction_from_name(instruction[0].as_str());
		let instr = match instr {
			Ok(i) => i,
			Err(e) => {
				println!("{} at line {}", e, line);
				return;
			}
		};

		instruction.retain(|x| x != "");
		let arg_count = opcodes[&instr].arguments_count;
		if instruction.len() != arg_count as usize + 1 {
			println!(
				"Wrong number of arguments for instruction {} at line {} (expected {} found {})",
				instr,
				line,
				arg_count,
				instruction.len() - 1
			);
			return;
		}

		let opc = instr.get_instruction_opc();
		let mut modes = Vec::new();
		let mut args: Vec<String> = Vec::new();

		for i in 0..arg_count as usize {
			let mut arg = instruction[i + 1].chars();
			let first_char = arg.next().unwrap();
			let arg: String = arg.collect();
			let mut arg_string: String = String::new();
			if first_char == '$' {
				modes.push(ParameterModes::Immediate);
			} else if first_char == '#' {
				modes.push(ParameterModes::Relative);
			} else if first_char.is_numeric() || first_char == '-' {
				modes.push(ParameterModes::Position);
				arg_string.push(first_char);
			} else {
				println!(
					"{}{} isn't a numeric argument (line: {}, arg: {})",
					first_char,
					arg,
					line,
					i + 1
				);
				return;
			}
			arg_string.push_str(arg.as_str());
			args.push(arg_string);
		}
		modes.reverse();

		let mut opcode_output = String::from("\n");
		let mut found_1 = false;
		for mode in modes {
			match mode {
				ParameterModes::Position => {
					if found_1 {
						opcode_output.push('0');
					}
				}
				ParameterModes::Immediate => {
					opcode_output.push('1');
					found_1 = true;
				}
				ParameterModes::Relative => {
					opcode_output.push('2');
					found_1 = true;
				}
				_ => panic!("Unknown mode"),
			}
		}
		if opc < 10 && found_1 {
			opcode_output.push('0');
		}
		opcode_output.push_str(opc.to_string().as_str());
		output.push(opcode_output);

		for arg in args {
			output.push(arg);
		}
	}

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
}
