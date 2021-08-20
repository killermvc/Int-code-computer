use icc::instructions::{Instructions, ParameterModes};
use std::collections::hash_map::HashMap;

pub fn get_mode_number_from_mode(mode: ParameterModes) -> (char, bool) {
	return match mode {
		ParameterModes::Position => ('0', false),
		ParameterModes::Immediate => ('1', true),
		ParameterModes::Relative => ('2', true),
		_ => panic!("Unknown mode"),
	};
}

pub const DEFAULT_MODE: ParameterModes = ParameterModes::Relative;

pub fn parse_argument(arg: &String) -> Result<(ParameterModes, String, bool), String> {
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

pub fn parse_instruction<'a>(instr: &'a String) -> (String, Option<String>) {
	let possible_tag: Vec<&str> = instr.split(":").collect();
	let mut tag: Option<String> = None;
	let instr = if possible_tag.len() > 1 {
		tag = Some(String::from(possible_tag[0]));
		possible_tag[1]
	} else {
		possible_tag[0]
	};
	(String::from(instr), tag)
}

#[derive(Debug, Clone)]
pub struct CodePosition {
	pub line: usize,
	pub column: usize,
	pub address: usize,
}

use std::fmt;

impl fmt::Display for CodePosition {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}:{}", self.line, self.column)
	}
}

pub enum CompileErrorType {
	ReservedTag(String),
	DuplicateTag(String, CodePosition),
	UnknownInstruction(String),
	WrongArgumentsCount(u8, usize),
	ArgumentParse(String),
	UndefinedTag(String),
	UnusedTag(String),
}

pub struct CompileError<'a> {
	pub error_type: CompileErrorType,
	pub file: &'a str,
	pub pos: CodePosition,
}

impl CompileError<'_> {
	pub fn new<'a>(error: CompileErrorType, file: &'a str, pos: CodePosition) -> CompileError {
		CompileError {
			error_type: error,
			file: file,
			pos: pos,
		}
	}
}

impl fmt::Display for CompileError<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let file_name = format!("{}:{} ", self.file, self.pos);
		match &self.error_type {
			CompileErrorType::ReservedTag(tag) => write!(
				f,
				"{}Tag <{}> is reserved by the compiler and can't be declared.",
				file_name, tag
			),
			CompileErrorType::DuplicateTag(tag, declared) => {
				write!(
					f,
					"{}Tag <{}> is already defined at: {}",
					file_name, tag, declared
				)
			}
			CompileErrorType::UnknownInstruction(instr) => {
				write!(f, "{}Unknown instruction <{}>", file_name, instr)
			}
			CompileErrorType::WrongArgumentsCount(expected, found) => write!(
				f,
				"{}Wrong number of arguments expected {} found {}",
				file_name, expected, found
			),
			CompileErrorType::ArgumentParse(e) => {
				write!(f, "{}Argument parsing failed: {}", file_name, e)
			}
			CompileErrorType::UndefinedTag(tag) => {
				write!(f, "{}undefined tag <{}>", file_name, tag)
			}
			CompileErrorType::UnusedTag(tag) => {
				write!(f, "{}Tag <{}> is never used.", file_name, tag)
			}
		}
	}
}

pub struct Assembler {
	input: Vec<Vec<String>>,
	tag_definitions: HashMap<String, CodePosition>,
	tag_uses: HashMap<String, Vec<CodePosition>>,
	current_line: usize,
	current_address: usize,
	filename: String,
}

impl Assembler {
	pub fn new(input: Vec<Vec<String>>, file: String) -> Assembler {
		Assembler {
			input: input,
			tag_definitions: HashMap::new(),
			tag_uses: HashMap::new(),
			current_line: 0,
			current_address: 0,
			filename: file,
		}
	}

	pub fn compile(&mut self) -> Result<Vec<String>, Vec<CompileError>> {
		let mut output: Vec<String> = Vec::new();
		let opcodes = Instructions::new();
		let mut errors_found: Vec<CompileError> = Vec::new();
		for instruction in self.input.iter_mut() {
			self.current_line += 1;
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
					line: self.current_line,
					column: 0,
					address: self.current_address,
				};
				if t == "data" {
					errors_found.push(CompileError::new(
						CompileErrorType::ReservedTag(String::from("data")),
						&self.filename[..],
						pos,
					));
					continue;
				} else if self.tag_definitions.contains_key(&t) {
					let defined_pos = &self.tag_definitions[&t.clone()];
					errors_found.push(CompileError::new(
						CompileErrorType::DuplicateTag(t, defined_pos.clone()),
						&self.filename[..],
						pos,
					));
					continue;
				}
				self.tag_definitions.insert(String::from(t), pos);
			}
			let instr = match Instructions::get_instruction_from_name(&instr[..]) {
				Some(i) => i,
				None => {
					errors_found.push(CompileError::new(
						CompileErrorType::UnknownInstruction(String::from(instr)),
						&self.filename[..],
						CodePosition {
							line: self.current_line,
							column: 0,
							address: self.current_address,
						},
					));
					continue;
				}
			};
			instruction.retain(|x| x != "");
			let arg_count = opcodes[&instr].arguments_count;
			if instruction.len() != arg_count as usize && arg_count != 0 {
				errors_found.push(CompileError::new(
					CompileErrorType::WrongArgumentsCount(arg_count, instruction.len()),
					&self.filename[..],
					CodePosition {
						line: self.current_line,
						column: 0,
						address: self.current_address,
					},
				));
			}
			let opc = instr.get_instruction_opc();
			let mut modes = Vec::new();
			let mut args: Vec<String> = Vec::new();
			for i in 0..arg_count as usize {
				self.current_address += 1;
				let arg = &instruction[i];
				let (mode, arg_string, is_tag) = match parse_argument(arg) {
					Ok(o) => o,
					Err(e) => {
						errors_found.push(CompileError::new(
							CompileErrorType::ArgumentParse(e),
							&self.filename[..],
							CodePosition {
								line: self.current_line,
								column: i + 1,
								address: self.current_address,
							},
						));
						continue;
					}
				};
				modes.push(mode);
				args.push(arg_string.clone());
				if is_tag {
					if !self.tag_uses.contains_key(&arg_string) {
						self.tag_uses.insert(arg_string.clone(), Vec::new());
					}
					let tags = self.tag_uses.get_mut(&arg_string).unwrap();
					tags.push(CodePosition {
						line: self.current_line,
						column: i + 2,
						address: self.current_address,
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
			self.current_address += 1;
		}
		self.tag_definitions.insert(
			String::from("data"),
			CodePosition {
				line: 0,
				column: 0,
				address: output.len(),
			},
		);
		for (tag, positions) in &self.tag_uses {
			if !self.tag_definitions.contains_key(tag) {
				for pos in positions {
					errors_found.push(CompileError {
						error_type: CompileErrorType::UndefinedTag(tag.clone()),
						file: &self.filename[..],
						pos: pos.clone(),
					});
				}
			} else {
				for pos in positions {
					let address = self.tag_definitions[tag].address;
					output[pos.address] = format!("{}", address);
				}
			}
		}
		for (tag, pos) in &self.tag_definitions {
			if !self.tag_uses.contains_key(tag) {
				errors_found.push(CompileError {
					error_type: CompileErrorType::UnusedTag(tag.clone()),
					file: &self.filename[..],
					pos: pos.clone(),
				});
			}
		}
		if errors_found.len() > 0 {
			Err(errors_found)
		} else {
			Ok(output)
		}
	}
}
