use std::collections::HashMap;

#[derive(Copy, Clone)]
pub enum ParameterModes {
    Position,
    Immediate,
    Relative,
}

#[derive(Clone)]
pub struct Instruction {
    pub arguments_count: u8,
    pub default_modes: Vec<ParameterModes>,
}

impl Instruction {
    pub fn new(arg_count: u8, default_mode: Vec<ParameterModes>) -> Instruction {
        Instruction {
            arguments_count: arg_count,
            default_modes: default_mode,
        }
    }
}

#[derive(Clone)]
pub struct Memory {
    data: Vec<i64>,
}

impl<'a> Memory {
    pub fn new(p_data: Vec<i64>) -> Memory {
        Memory { data: p_data }
    }

    pub fn read(&self, index: usize) -> i64 {
        self.data[index]
    }

    pub fn write(&mut self, index: usize, value: i64) {
        self.data[index] = value;
    }
}

#[derive(PartialEq, Hash, Eq, Debug)]
enum Instructions {
    ADD,
    MUL,
    IN,
    OUT,
    JMP,
    JMPF,
    LESS,
    EQ,
    HLT,
}

pub struct IntCodeProgram {
    instruction_pointer: usize,
    memory: Memory,
    base: usize,
    opcodes: HashMap<Instructions, Instruction>,
    input: Vec<i64>,
    next_input: usize,
    output: Vec<i64>,
}

impl IntCodeProgram {
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
            99 => Instructions::HLT,
            _ => panic!("Unknown instruction {}", instr_opc),
        }
    }

    pub fn new(p_memory: Memory, input: Vec<i64>) -> IntCodeProgram {
        let opcodes = HashMap::new();
        let mut program = IntCodeProgram {
            instruction_pointer: 0,
            memory: p_memory,
            opcodes: opcodes,
            base: 0,
            input: input,
            next_input: 0,
            output: Vec::new(),
        };
        program.initialize_opcodes();
        program
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
                2 => modes_arr[i] = ParameterModes::Relative,
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
        println!("pointer: {}", self.instruction_pointer - 1);
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
                        .read(self.base + self.memory.read(self.instruction_pointer) as usize)
                }
            }
            self.instruction_pointer += 1;
        }
        (instruction, args)
    }

    fn execute_instruction(&mut self, instr: Instructions, args: [i64; 3]) {
        println!("Executing instruction {:?} with args {:?}", instr, args);
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
                value = self.input[self.next_input];
                self.next_input += 1;
                store_adress = args[0];
            }
            Instructions::OUT => {
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
            _ => panic!("Unknown instruction (or halt) in execute_instruction"),
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
