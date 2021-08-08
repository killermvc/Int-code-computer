use std::collections::HashMap;

#[derive(Clone)]
pub struct Memory {
    initial_data: Vec<i64>,
    expanded_memory: HashMap<usize, i64>,
}

impl<'a> Memory {
    pub fn new(p_data: Vec<i64>) -> Memory {
        Memory {
            initial_data: p_data,
            expanded_memory: HashMap::new(),
        }
    }

    pub fn read(&self, index: usize) -> i64 {
        if index >= self.initial_data.len() {
            if !self.expanded_memory.contains_key(&index) {
                0
            } else {
                self.expanded_memory[&index]
            }
        } else {
            self.initial_data[index]
        }
    }

    pub fn write(&mut self, index: usize, value: i64) {
        if index >= self.initial_data.len() {
            self.expanded_memory.insert(index, value);
        } else {
            self.initial_data[index] = value;
        }
    }
}

pub mod instructions_module {
    use std::collections::HashMap;
    use std::fmt;

    #[derive(Copy, Clone, PartialEq)]
    pub enum ParameterModes {
        Position,
        Immediate,
        Relative,
        RelativeImmediate,
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

    #[derive(PartialEq, Hash, Eq, Debug)]
    pub enum Instructions {
        ADD,
        MUL,
        IN,
        OUT,
        JMP,
        JMPF,
        LESS,
        EQ,
        ARB,
        MOV,
        GRT,
        HLT,
    }

    impl fmt::Display for Instructions {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl Instructions {
        pub fn get_instruction_from_opc(instr_opc: usize) -> Result<Instructions, String> {
            match instr_opc {
                1 => Ok(Instructions::ADD),
                2 => Ok(Instructions::MUL),
                3 => Ok(Instructions::IN),
                4 => Ok(Instructions::OUT),
                5 => Ok(Instructions::JMP),
                6 => Ok(Instructions::JMPF),
                7 => Ok(Instructions::LESS),
                8 => Ok(Instructions::EQ),
                9 => Ok(Instructions::ARB),
                10 => Ok(Instructions::MOV),
                11 => Ok(Instructions::GRT),
                99 => Ok(Instructions::HLT),
                _ => Err(format!("Unknown instruction {}", instr_opc)),
            }
        }

        pub fn get_instruction_opc(&self) -> usize {
            match self {
                Instructions::ADD => 1,
                Instructions::MUL => 2,
                Instructions::IN => 3,
                Instructions::OUT => 4,
                Instructions::JMP => 5,
                Instructions::JMPF => 6,
                Instructions::LESS => 7,
                Instructions::EQ => 8,
                Instructions::ARB => 9,
                Instructions::MOV => 10,
                Instructions::GRT => 11,
                Instructions::HLT => 99,
            }
        }

        pub fn get_instruction_from_name(instr_name: &str) -> Result<Instructions, String> {
            match instr_name.to_lowercase().as_str() {
                "add" => Ok(Instructions::ADD),
                "mul" => Ok(Instructions::MUL),
                "in" => Ok(Instructions::IN),
                "out" => Ok(Instructions::OUT),
                "jmp" => Ok(Instructions::JMP),
                "jmpf" => Ok(Instructions::JMPF),
                "less" => Ok(Instructions::LESS),
                "eq" => Ok(Instructions::EQ),
                "arb" => Ok(Instructions::ARB),
                "mov" => Ok(Instructions::MOV),
                "grt" => Ok(Instructions::GRT),
                "hlt" => Ok(Instructions::HLT),
                _ => Err(format!("Unknown instruction {}", instr_name)),
            }
        }

        pub fn new() -> HashMap<Instructions, Instruction> {
            let mut opcodes = HashMap::new();
            opcodes.insert(
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
            opcodes.insert(
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
            opcodes.insert(
                Instructions::IN,
                Instruction::new(1, vec![ParameterModes::Immediate]),
            );
            opcodes.insert(
                Instructions::OUT,
                Instruction::new(1, vec![ParameterModes::Position]),
            );
            opcodes.insert(
                Instructions::JMP,
                Instruction::new(2, vec![ParameterModes::Position, ParameterModes::Position]),
            );
            opcodes.insert(
                Instructions::JMPF,
                Instruction::new(2, vec![ParameterModes::Position, ParameterModes::Position]),
            );
            opcodes.insert(
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
            opcodes.insert(
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
            opcodes.insert(
                Instructions::ARB,
                Instruction::new(1, vec![ParameterModes::Position]),
            );
            opcodes.insert(
                Instructions::MOV,
                Instruction::new(2, vec![ParameterModes::Position, ParameterModes::Immediate]),
            );
            opcodes.insert(
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
            opcodes.insert(Instructions::HLT, Instruction::new(0, vec![]));

            opcodes
        }
    }
}
