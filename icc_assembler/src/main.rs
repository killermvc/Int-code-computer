extern crate regex;
use icc::instructions::{Instructions, ParameterModes};
use regex::Regex;
use std::collections::hash_map::HashMap;
use std::fs;
use std::io::Write;

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

fn parse_instruction<'a>(instr: &'a String) -> (String, Option<String>) {
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
struct CodePosition {
    line: usize,
    column: usize,
    address: usize,
}

use std::fmt;

impl fmt::Display for CodePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

enum CompileErrorType<'a> {
    ReservedTag(&'a str),
    DuplicateTag(String, &'a CodePosition),
    UnknownInstruction(String),
    WrongArgumentsCount(u8, usize),
    ArgumentParse(String),
    UndefinedTag(String),
    UnusedTag(String),
}

struct CompileError<'a> {
    error_type: CompileErrorType<'a>,
    file: &'a str,
    pos: CodePosition,
}

impl CompileError<'_> {
    fn new<'a>(error: CompileErrorType<'a>, file: &'a str, pos: CodePosition) -> CompileError<'a> {
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
                write!(f, "{}Error while parsing argument: {}", file_name, e)
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
    let mut errors_found: Vec<CompileError> = Vec::new();
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
                errors_found.push(CompileError::new(
                    CompileErrorType::ReservedTag("data"),
                    filename,
                    pos,
                ));
                return;
            }
            if tag_definitions.contains_key(&t) {
                let defined_pos = &tag_definitions[&t.clone()];
                errors_found.push(CompileError::new(
                    CompileErrorType::DuplicateTag(t, defined_pos),
                    filename,
                    pos,
                ));
                return;
            }
            tag_definitions.insert(String::from(t), pos);
        }

        let instr = match Instructions::get_instruction_from_name(&instr[..]) {
            Some(i) => i,
            None => {
                errors_found.push(CompileError::new(
                    CompileErrorType::UnknownInstruction(String::from(instr)),
                    filename,
                    CodePosition {
                        line: line,
                        column: 0,
                        address: current_address,
                    },
                ));
                return;
            }
        };

        instruction.retain(|x| x != "");
        let arg_count = opcodes[&instr].arguments_count;
        if instruction.len() != arg_count as usize && arg_count != 0 {
            errors_found.push(CompileError::new(
                CompileErrorType::WrongArgumentsCount(arg_count, instruction.len()),
                filename,
                CodePosition {
                    line: line,
                    column: 0,
                    address: current_address,
                },
            ));
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
                    errors_found.push(CompileError::new(
                        CompileErrorType::ArgumentParse(e),
                        filename,
                        CodePosition {
                            line: line,
                            column: i + 1,
                            address: current_address,
                        },
                    ));
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

    for (tag, positions) in &tag_uses {
        if !tag_definitions.contains_key(tag) {
            for pos in positions {
                errors_found.push(CompileError {
                    error_type: CompileErrorType::UndefinedTag(tag.clone()),
                    file: &filename,
                    pos: pos.clone(),
                });
            }
        } else {
            for pos in positions {
                let address = tag_definitions[tag].address;
                output[pos.address] = format!("{}", address);
            }
        }
    }

    for (tag, pos) in tag_definitions {
        if !tag_uses.contains_key(&tag) {
            errors_found.push(CompileError {
                error_type: CompileErrorType::UnusedTag(tag),
                file: &filename,
                pos: pos,
            });
        }
    }

    if errors_found.len() > 0 {
        let mut error_count = 0;
        for error in errors_found {
            if error_count > 50 {
                break;
            }
            println!("{}", error);
            error_count += 1;
        }
        println!("Build failed with {} errors.", error_count);
        return;
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
