use crate::lib::{Instruction, Instructions, Memory, ParameterModes};
use std::collections::HashMap;

type OutputHandle = fn(i64);
type InputHandle = fn() -> i64;

pub struct IntCodeProgram {
	instruction_pointer: usize,
	memory: Memory,
	base: i64,
	opcodes: HashMap<Instructions, Instruction>,
	input: Vec<i64>,
	next_input: usize,
	output: Vec<i64>,
	output_handle: OutputHandle,
	input_handle: InputHandle,
}

impl IntCodeProgram {
	pub fn new(
		p_memory: Memory,
		input: Vec<i64>,
		output_handle: OutputHandle,
		input_handle: InputHandle,
	) -> IntCodeProgram {
		let opcodes = HashMap::new();
		let mut program = IntCodeProgram {
			instruction_pointer: 0,
			memory: p_memory,
			opcodes: opcodes,
			base: 0,
			input: input,
			next_input: 0,
			output: Vec::new(),
			output_handle: output_handle,
			input_handle: input_handle,
		};
		program.initialize_opcodes();
		program
	}

	fn initialize_opcodes(&mut self) {
		self.opcodes.insert(
			Instructions::ADD,
			Instruction::new(
				3,
				vec![
					ParameterModes::Position,
					ParameterModes::Position,
					ParameterModes::Immediate,
				],
			),
		);
		self.opcodes.insert(
			Instructions::MUL,
			Instruction::new(
				3,
				vec![
					ParameterModes::Position,
					ParameterModes::Position,
					ParameterModes::Immediate,
				],
			),
		);
		self.opcodes.insert(
			Instructions::IN,
			Instruction::new(1, vec![ParameterModes::Immediate]),
		);
		self.opcodes.insert(
			Instructions::OUT,
			Instruction::new(1, vec![ParameterModes::Position]),
		);
		self.opcodes.insert(
			Instructions::JMP,
			Instruction::new(2, vec![ParameterModes::Position, ParameterModes::Position]),
		);
		self.opcodes.insert(
			Instructions::JMPF,
			Instruction::new(2, vec![ParameterModes::Position, ParameterModes::Position]),
		);
		self.opcodes.insert(
			Instructions::LESS,
			Instruction::new(
				3,
				vec![
					ParameterModes::Position,
					ParameterModes::Position,
					ParameterModes::Immediate,
				],
			),
		);
		self.opcodes.insert(
			Instructions::EQ,
			Instruction::new(
				3,
				vec![
					ParameterModes::Position,
					ParameterModes::Position,
					ParameterModes::Immediate,
				],
			),
		);
		self.opcodes.insert(
			Instructions::ARB,
			Instruction::new(1, vec![ParameterModes::Position]),
		);
		self.opcodes.insert(
			Instructions::MOV,
			Instruction::new(2, vec![ParameterModes::Position, ParameterModes::Immediate]),
		);
		self.opcodes.insert(
			Instructions::GRT,
			Instruction::new(
				3,
				vec![
					ParameterModes::Position,
					ParameterModes::Position,
					ParameterModes::Immediate,
				],
			),
		);
		self.opcodes
			.insert(Instructions::HLT, Instruction::new(0, vec![]));
	}

	fn get_instruction_from_opc(instr_opc: usize) -> Instructions {
		match instr_opc {
			1 => Instructions::ADD,
			2 => Instructions::MUL,
			3 => Instructions::IN,
			4 => Instructions::OUT,
			5 => Instructions::JMP,
			6 => Instructions::JMPF,
			7 => Instructions::LESS,
			8 => Instructions::EQ,
			9 => Instructions::ARB,
			10 => Instructions::MOV,
			11 => Instructions::GRT,
			99 => Instructions::HLT,
			_ => panic!("Unknown instruction {}", instr_opc),
		}
	}

	fn parse_instruction(
		&self,
		instruction_opc: i64,
	) -> (Instructions, &Instruction, Vec<ParameterModes>) {
		let instruction = instruction_opc % 100;
		let instruction = IntCodeProgram::get_instruction_from_opc(instruction as usize);
		let instruction_data = &self.opcodes[&instruction];
		let mut modes = instruction_opc / 100;
		let mut modes_arr = instruction_data.default_modes.clone();
		let mut i = 0;

		while modes != 0 {
			match modes % 10 {
				0 => modes_arr[i] = ParameterModes::Position,
				1 => modes_arr[i] = ParameterModes::Immediate,
				2 => {
					if modes_arr[i] == ParameterModes::Immediate {
						modes_arr[i] = ParameterModes::RelativeImmediate;
					} else {
						modes_arr[i] = ParameterModes::Relative;
					}
				}
				_ => panic!("Unknown mode"),
			}
			modes = modes / 10;
			i += 1;
		}
		(instruction, instruction_data, modes_arr)
	}

	fn get_next_instruction(&mut self) -> (Instructions, [i64; 3]) {
		let instruction = self.memory.read(self.instruction_pointer);
		self.instruction_pointer += 1;
		let (instruction, instruction_data, modes) = self.parse_instruction(instruction);
		let mut args = [0; 3];
		let arg_count = instruction_data.arguments_count as usize;
		for i in 0usize..arg_count {
			match modes[i] {
				ParameterModes::Position => {
					args[i] = self
						.memory
						.read(self.memory.read(self.instruction_pointer) as usize)
				}
				ParameterModes::Immediate => args[i] = self.memory.read(self.instruction_pointer),
				ParameterModes::Relative => {
					args[i] = self
						.memory
						.read((self.base + self.memory.read(self.instruction_pointer)) as usize);
				}
				ParameterModes::RelativeImmediate => {
					args[i] = self.base + self.memory.read(self.instruction_pointer);
				}
			}
			self.instruction_pointer += 1;
		}
		(instruction, args)
	}

	fn execute_instruction(&mut self, instr: Instructions, args: [i64; 3]) {
		let store_adress;
		let mut value = 0;
		match instr {
			Instructions::ADD => {
				value = args[0] + args[1];
				store_adress = args[2];
			}
			Instructions::MUL => {
				value = args[0] * args[1];
				store_adress = args[2];
			}
			Instructions::IN => {
				if self.next_input >= self.input.len() {
					let handle = self.input_handle;
					value = handle();
				} else {
					value = self.input[self.next_input];
					self.next_input += 1;
				}
				store_adress = args[0];
			}
			Instructions::OUT => {
				let handle = self.output_handle;
				handle(args[0]);
				self.output.push(args[0]);
				store_adress = -1;
			}
			Instructions::JMP => {
				if args[0] != 0 {
					self.instruction_pointer = args[1] as usize;
				}
				store_adress = -1;
			}
			Instructions::JMPF => {
				if args[0] == 0 {
					self.instruction_pointer = args[1] as usize;
				}
				store_adress = -1;
			}
			Instructions::LESS => {
				if args[0] < args[1] {
					value = 1;
				} else {
					value = 0;
				}
				store_adress = args[2];
			}
			Instructions::EQ => {
				if args[0] == args[1] {
					value = 1;
				} else {
					value = 0;
				}
				store_adress = args[2];
			}
			Instructions::ARB => {
				self.base += args[0];
				store_adress = -1;
			}
			Instructions::MOV => {
				value = args[0];
				store_adress = args[1];
			}
			Instructions::GRT => {
				if args[0] > args[1] {
					value = 1;
				} else {
					value = 0;
				}
				store_adress = args[2];
			}
			Instructions::HLT => panic!("Hlt in execute_instruction"),
		}
		if store_adress < 0 {
			return;
		}
		self.memory.write(store_adress as usize, value);
	}

	pub fn run(mut self) -> Vec<i64> {
		let (mut instruction, mut args) = self.get_next_instruction();
		while instruction != Instructions::HLT {
			self.execute_instruction(instruction, args);
			let res = self.get_next_instruction();
			instruction = res.0;
			args = res.1;
		}
		self.output
	}
}
