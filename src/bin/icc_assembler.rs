use lib::instructions_module::{Instructions, ParameterModes};
use std::collections::hash_map::HashMap;
use std::fs;
use std::io::Write;
extern crate regex;
use regex::Regex;

fn get_mode_number_from_mode(mode: ParameterModes) -> (char, bool) {
	return match mode {
		ParameterModes::Position => ('0', false),
		ParameterModes::Immediate => ('1', true),
		ParameterModes::Relative => ('2', true),
		_ => panic!("Unknown mode"),
	};
}

fn parse_argument(arg: &String) -> Result<(ParameterModes, String, bool), String> {
	let mut char_iter = arg.chars();
	let first_char = match char_iter.next() {
		Some(c) => c,
		None => return Err(String::from("Empty argument")),
	};
	let mode;
	let mut is_tag: bool = false;
	if first_char == '$' {
		mode = ParameterModes::Immediate;
	} else if first_char == '#' {
		mode = ParameterModes::Relative;
	} else {
		mode = ParameterModes::Position;
		if !first_char.is_numeric() && first_char != '-' {
			is_tag = true;
		}
	}

	let second_char;
	second_char = char_iter.next();

	let mut output = String::new();
	if mode == ParameterModes::Position {
		output.push(first_char);
	}
	if let Some(c) = second_char {
		if !is_tag && c != '-' && !c.is_numeric() {
			is_tag = true;
		}
		output.push(c);
	}
	let rest: String = char_iter.collect();
	output.push_str(rest.as_str());

	Ok((mode, output, is_tag))
}

fn parase_instruction<'a>(instr: &'a String) -> (&'a str, Option<&'a str>) {
	let possible_tag: Vec<&str> = instr.split(":").collect();
	let mut tag: Option<&str> = None;
	let instr = if possible_tag.len() > 1 {
		tag = Some(possible_tag[0]);
		possible_tag[1]
	} else {
		possible_tag[0]
	};
	(instr, tag)
}

struct TagPosition {
	line: usize,
	column: usize,
	address: usize,
}

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
		.map(|arg| arg.split(" ").map(|arg| arg.replace("\r", "")).collect())
		.collect();

	let mut output: Vec<String> = Vec::new();
	let opcodes = Instructions::new();
	let mut line: usize = 0;
	let mut tag_definitions: HashMap<String, usize> = HashMap::new();
	let mut tag_uses: HashMap<String, TagPosition> = HashMap::new();
	let mut current_address: usize = 0;
	for mut instruction in input {
		line += 1;
		if instruction[0].is_empty() || instruction[0].find("\r").unwrap_or(1) == 0 {
			continue;
		}

		let (instr, tag_option) = parase_instruction(&instruction[0]);
		if let Some(t) = tag_option {
			tag_definitions.insert(String::from(t), current_address);
		}

		let instr = Instructions::get_instruction_from_name(instr);
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
			current_address += 1;
			let arg = &instruction[i + 1];
			let (mode, arg_string, is_tag) = match parse_argument(arg) {
				Ok(o) => o,
				Err(e) => {
					println!("Error parsing argument {} at line {}: {}", i + 1, line, e);
					return;
				}
			};
			modes.push(mode);
			args.push(arg_string.clone());
			if is_tag {
				tag_uses.insert(
					arg_string,
					TagPosition {
						line: line,
						column: i + 2,
						address: current_address,
					},
				);
			}
		}
		modes.reverse();

		let mut opcode_output = String::from("\n");
		let mut found_1 = false;
		for mode in modes {
			let (ch, found) = get_mode_number_from_mode(mode);
			if found {
				found_1 = found;
			}
			if found_1 || ch != '0' {
				opcode_output.push(ch);
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
		current_address += 1;
	}

	for (tag, pos) in tag_uses {
		if !tag_definitions.contains_key(&tag) {
			println!(
				"Undefined tag {} at (address: {}, line:{}, column: {})",
				tag, pos.address, pos.line, pos.column
			);
			return;
		}
		let address = tag_definitions[&tag];
		output[pos.address] = format!("{}", address);
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
