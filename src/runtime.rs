use crate::lexer::{JumpTable, Token};
use std::io::{stdin, Error, Read};

pub struct State {
    memory: Vec<u8>,
    memory_pointer: usize,
    instruction_pointer: usize,
}

#[derive(Debug)]
pub enum ExecutionError {
    EndOfInstructions,
    PointerUnderflow(usize),
    UndefinedJumpTarget(usize),
    InputError(usize, Error),
}

impl State {
    pub fn new() -> Self {
        Self {
            memory: vec![0],
            memory_pointer: 0,
            instruction_pointer: 0,
        }
    }

    pub fn can_execute(&self, tokens: &[Token]) -> bool {
        self.instruction_pointer < tokens.len()
    }

    pub fn execute_current_instruction(
        &mut self,
        tokens: &[Token],
        jump_table: &JumpTable,
    ) -> Result<(), ExecutionError> {
        if !self.can_execute(tokens) {
            return Err(ExecutionError::EndOfInstructions);
        }

        match tokens[self.instruction_pointer] {
            Token::Increment => self.execute_increment(),
            Token::Decrement => self.execute_decrement(),
            Token::PointerIncrement => self.execute_pointer_increment(),
            Token::PointerDecrement => return self.execute_pointer_decrement(),
            Token::LoopStart => return self.execute_loop_start(jump_table),
            Token::LoopEnd => return self.execute_loop_end(jump_table),
            Token::Input => return self.execute_input(),
            Token::Output => self.execute_output(),
        }

        Ok(())
    }

    fn execute_increment(&mut self) {
        self.memory[self.memory_pointer] += 1;
        self.instruction_pointer += 1;
    }

    fn execute_decrement(&mut self) {
        self.memory[self.memory_pointer] -= 1;
        self.instruction_pointer += 1;
    }

    fn execute_pointer_increment(&mut self) {
        self.memory_pointer += 1;
        if self.memory.len() == self.memory_pointer {
            self.memory.push(0);
        }
        self.instruction_pointer += 1;
    }

    fn execute_pointer_decrement(&mut self) -> Result<(), ExecutionError> {
        if self.memory_pointer == 0 {
            return Err(ExecutionError::PointerUnderflow(self.instruction_pointer));
        }
        self.memory_pointer -= 1;
        self.instruction_pointer += 1;
        Ok(())
    }

    fn execute_loop_start(&mut self, jump_table: &JumpTable) -> Result<(), ExecutionError> {
        match self.memory[self.memory_pointer] {
            0 => match jump_table.resolve(&self.instruction_pointer) {
                Some(x) => self.instruction_pointer = *x + 1,
                None => {
                    return Err(ExecutionError::UndefinedJumpTarget(
                        self.instruction_pointer,
                    ))
                }
            },
            _ => self.instruction_pointer += 1,
        }
        Ok(())
    }

    fn execute_loop_end(&mut self, jump_table: &JumpTable) -> Result<(), ExecutionError> {
        match jump_table.resolve(&self.instruction_pointer) {
            Some(x) => self.instruction_pointer = *x,
            None => {
                return Err(ExecutionError::UndefinedJumpTarget(
                    self.instruction_pointer,
                ))
            }
        };
        Ok(())
    }

    fn execute_input(&mut self) -> Result<(), ExecutionError> {
        let mut buffer = [0];
        match stdin().read_exact(&mut buffer) {
            Ok(()) => self.memory[self.memory_pointer] = buffer[0],
            Err(e) => return Err(ExecutionError::InputError(self.instruction_pointer, e)),
        }
        Ok(())
    }

    fn execute_output(&mut self) {
        print!("{}", self.memory[self.memory_pointer] as char);
        self.instruction_pointer += 1;
    }
}
