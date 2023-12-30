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
        let (value, _carry) = self.memory[self.memory_pointer].overflowing_add(1);
        self.memory[self.memory_pointer] = value;
        self.instruction_pointer += 1;
    }

    fn execute_decrement(&mut self) {
        let (value, _borrow) = self.memory[self.memory_pointer].overflowing_sub(1);
        self.memory[self.memory_pointer] = value;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_execute_true_when_tokens_left() {
        let state = State::new();
        assert_eq!(state.can_execute(&[Token::Increment]), true);
    }

    #[test]
    fn can_execute_false_when_no_tokens_left() {
        let state = State::new();
        assert_eq!(state.can_execute(&[]), false);
    }

    #[test]
    fn error_on_execute_finished_program() {
        let mut state = State::new();
        let tokens = [];
        let jump_table = JumpTable::from(&tokens).unwrap();

        let result = state.execute_current_instruction(&tokens, &jump_table);

        assert!(result.is_err());
        match result.unwrap_err() {
            ExecutionError::EndOfInstructions => assert!(true),
            _ => assert!(false),
        };
    }

    #[test]
    fn increments_adds_one_to_current() {
        let mut state = State::new();
        let tokens = [Token::Increment];
        let jump_table = JumpTable::from(&tokens).unwrap();

        let result = state.execute_current_instruction(&tokens, &jump_table);

        assert!(result.is_ok());
        assert_eq!(state.memory[0], 1);
        assert_eq!(state.instruction_pointer, 1);
    }

    #[test]
    fn increments_overflows() {
        let mut state = State::new();
        let tokens = [Token::Increment];
        let jump_table = JumpTable::from(&tokens).unwrap();

        state.memory[0] = 255;
        let result = state.execute_current_instruction(&tokens, &jump_table);

        assert!(result.is_ok());
        assert_eq!(state.memory[0], 0);
        assert_eq!(state.instruction_pointer, 1);
    }

    #[test]
    fn decrements_subtracts_one_to_current() {
        let mut state = State::new();
        let tokens = [Token::Decrement];
        let jump_table = JumpTable::from(&tokens).unwrap();

        state.memory[0] = 10;
        let result = state.execute_current_instruction(&tokens, &jump_table);

        assert!(result.is_ok());
        assert_eq!(state.memory[0], 9);
        assert_eq!(state.instruction_pointer, 1);
    }

    #[test]
    fn decrements_underflows() {
        let mut state = State::new();
        let tokens = [Token::Decrement];
        let jump_table = JumpTable::from(&tokens).unwrap();

        let result = state.execute_current_instruction(&tokens, &jump_table);

        assert!(result.is_ok());
        assert_eq!(state.memory[0], 255);
        assert_eq!(state.instruction_pointer, 1);
    }

    #[test]
    fn pointer_increment_advances_memory_pointer() {
        let mut state = State::new();
        let tokens = [Token::PointerIncrement];
        let jump_table = JumpTable::from(&tokens).unwrap();

        let result = state.execute_current_instruction(&tokens, &jump_table);

        assert!(result.is_ok());
        assert_eq!(state.memory_pointer, 1);
        assert_eq!(state.memory.len(), 2);
        assert_eq!(state.instruction_pointer, 1);
    }

    #[test]
    fn pointer_decrement_moves_back_memory_pointer() {
        let mut state = State::new();
        let tokens = [Token::PointerDecrement];
        let jump_table = JumpTable::from(&tokens).unwrap();

        state.memory_pointer = 2;
        state.memory = vec![0; 3];
        let result = state.execute_current_instruction(&tokens, &jump_table);

        assert!(result.is_ok());
        assert_eq!(state.memory_pointer, 1);
        assert_eq!(state.memory.len(), 3);
        assert_eq!(state.instruction_pointer, 1);
    }

    #[test]
    fn pointer_decrement_errors_on_underflow() {
        let mut state = State::new();
        let tokens = [Token::PointerDecrement];
        let jump_table = JumpTable::from(&tokens).unwrap();

        let result = state.execute_current_instruction(&tokens, &jump_table);

        assert!(result.is_err());
        match result.unwrap_err() {
            ExecutionError::PointerUnderflow(x) => assert_eq!(x, 0),
            _ => assert!(false),
        };
    }

    #[test]
    fn loop_start_enters_loop_on_none_zero() {
        let mut state = State::new();
        let tokens = [
            Token::LoopStart,
            Token::Increment,
            Token::LoopEnd,
            Token::Decrement,
        ];
        let jump_table = JumpTable::from(&tokens).unwrap();

        state.memory[0] = 1;
        let result = state.execute_current_instruction(&tokens, &jump_table);

        assert!(result.is_ok());
        assert_eq!(state.instruction_pointer, 1);
    }

    #[test]
    fn loop_start_jumps_over_loop_on_zero() {
        let mut state = State::new();
        let tokens = [
            Token::LoopStart,
            Token::Increment,
            Token::LoopEnd,
            Token::Decrement,
        ];
        let jump_table = JumpTable::from(&tokens).unwrap();

        let result = state.execute_current_instruction(&tokens, &jump_table);

        assert!(result.is_ok());
        assert_eq!(state.instruction_pointer, 3);
    }

    #[test]
    fn loop_start_errors_on_invalid_jump_table_on_zero() {
        let mut state = State::new();
        let tokens = [];
        let jump_table = JumpTable::from(&tokens).unwrap();
        let tokens = [
            Token::LoopStart,
            Token::Increment,
            Token::LoopEnd,
            Token::Decrement,
        ];

        let result = state.execute_current_instruction(&tokens, &jump_table);

        assert!(result.is_err());
        match result.unwrap_err() {
            ExecutionError::UndefinedJumpTarget(x) => assert_eq!(x, 0),
            _ => assert!(false),
        };
    }

    #[test]
    fn loop_end_jumps_back_to_start() {
        let mut state = State::new();
        let tokens = [
            Token::LoopStart,
            Token::Increment,
            Token::LoopEnd,
            Token::Decrement,
        ];
        let jump_table = JumpTable::from(&tokens).unwrap();

        state.instruction_pointer = 2;
        let result = state.execute_current_instruction(&tokens, &jump_table);

        assert!(result.is_ok());
        assert_eq!(state.instruction_pointer, 0);
    }

    #[test]
    fn loop_end_errors_on_invalid_jump_table() {
        let mut state = State::new();
        let tokens = [];
        let jump_table = JumpTable::from(&tokens).unwrap();
        let tokens = [
            Token::LoopStart,
            Token::Increment,
            Token::LoopEnd,
            Token::Decrement,
        ];

        state.instruction_pointer = 2;
        let result = state.execute_current_instruction(&tokens, &jump_table);

        assert!(result.is_err());
        match result.unwrap_err() {
            ExecutionError::UndefinedJumpTarget(x) => assert_eq!(x, 2),
            _ => assert!(false),
        };
    }
}
