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

const DEFAULT_MODE: ParameterModes = ParameterModes::Relative;

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
		mode = ParameterModes::Position;
	} else {
		mode = DEFAULT_MODE;
		if !first_char.is_numeric() && first_char != '-' {
			is_tag = true;
		}
	}

	let second_char;
	second_char = char_iter.next();

	let mut output = String::new();
	if mode == DEFAULT_MODE {
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

fn parse_instruction<'a>(instr: &'a String) -> (&'a str, Option<&'a str>) {
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

#[derive(Debug)]
struct CodePosition {
	line: usize,
	column: usize,
	address: usize,
}

use std::fmt;

impl fmt::Display for CodePosition {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "({}:{})", self.line, self.column)
	}
}

enum CompileError<'a> {
	ReservedTag(&'a str),
	DuplicateTag(&'a str, &'a CodePosition),
	UnknownInstruction(&'a str),
	WrongArgumentsCount(u8, usize),
	ArgumentParse(String),
	UndefinedTag(&'a str),
}

fn print_error(error: CompileError, filename: &str, pos: CodePosition) {
	print!("{}{}: ", filename, pos);
	match error {
		CompileError::ReservedTag(tag) => println!(
			"Tag <{}> is reserved by the compiler and can't be declared.",
			tag
		),
		CompileError::DuplicateTag(tag, declared) => {
			println!("Tag <{}> was already defined at: {}", tag, declared)
		}
		CompileError::UnknownInstruction(instr) => println!("Unknown instruction <{}>", instr),
		CompileError::WrongArgumentsCount(expected, found) => println!(
			"Wrong number of arguments expected {} found {}",
			expected, found
		),
		CompileError::ArgumentParse(e) => println!("Error while parsing argument: {}", e),
		CompileError::UndefinedTag(tag) => println!("Tag <{}> was never declared.", tag),
	}
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
		.map(|arg| {
			arg.split(",")
				.map(|arg| arg.trim_start().trim_end().replace("\r", ""))
				.collect()
		})
		.collect();

	let mut output: Vec<String> = Vec::new();
	let opcodes = Instructions::new();
	let mut line: usize = 0;
	let mut tag_definitions: HashMap<String, CodePosition> = HashMap::new();
	let mut tag_uses: HashMap<String, Vec<CodePosition>> = HashMap::new();
	let mut current_address: usize = 0;
	for mut instruction in input {
		line += 1;
		let instr: Vec<String> = instruction[0]
			.split(" ")
			.map(|arg| String::from(arg))
			.collect();
		if instr.len() > 1 {
			instruction[0] = String::from(instr[1].clone());
		}
		if instr[0].is_empty() || instr[0].find("\r").unwrap_or(1) == 0 {
			continue;
		}

		let (instr, tag_option) = parse_instruction(&instr[0]);
		if let Some(t) = tag_option {
			let pos = CodePosition {
				line: line,
				column: 0,
				address: current_address,
			};
			if t == "data" {
				print_error(CompileError::ReservedTag("data"), filename, pos);
				return;
			}
			if tag_definitions.contains_key(t) {
				print_error(
					CompileError::DuplicateTag(t, &tag_definitions[t]),
					filename,
					pos,
				);
				return;
			}
			tag_definitions.insert(String::from(t), pos);
		}

		let instr = match Instructions::get_instruction_from_name(instr) {
			Some(i) => i,
			None => {
				print_error(
					CompileError::UnknownInstruction(instr),
					filename,
					CodePosition {
						line: line,
						column: 0,
						address: current_address,
					},
				);
				return;
			}
		};

		instruction.retain(|x| x != "");
		let arg_count = opcodes[&instr].arguments_count;
		if instruction.len() != arg_count as usize && arg_count != 0 {
			print_error(
				CompileError::WrongArgumentsCount(arg_count, instruction.len()),
				filename,
				CodePosition {
					line: line,
					column: 0,
					address: current_address,
				},
			);
			return;
		}

		let opc = instr.get_instruction_opc();
		let mut modes = Vec::new();
		let mut args: Vec<String> = Vec::new();

		for i in 0..arg_count as usize {
			current_address += 1;
			let arg = &instruction[i];
			let (mode, arg_string, is_tag) = match parse_argument(arg) {
				Ok(o) => o,
				Err(e) => {
					print_error(
						CompileError::ArgumentParse(e),
						filename,
						CodePosition {
							line: line,
							column: i + 1,
							address: current_address,
						},
					);
					return;
				}
			};
			modes.push(mode);
			args.push(arg_string.clone());
			if is_tag {
				if !tag_uses.contains_key(&arg_string) {
					tag_uses.insert(arg_string.clone(), Vec::new());
				}
				let tags = tag_uses.get_mut(&arg_string).unwrap();
				tags.push(CodePosition {
					line: line,
					column: i + 2,
					address: current_address,
				});
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

	tag_definitions.insert(
		String::from("data"),
		CodePosition {
			line: 0,
			column: 0,
			address: output.len(),
		},
	);

	for (tag, positions) in tag_uses {
		if !tag_definitions.contains_key(&tag) {
			for pos in positions {
				print_error(CompileError::UndefinedTag(&tag[..]), filename, pos);
			}
			return;
		}

		for pos in positions {
			let address = tag_definitions[&tag].address;
			output[pos.address] = format!("{}", address);
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
	println!("Succesfully compiled {} to {}", filename, output_name);
}
