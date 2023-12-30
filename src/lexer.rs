use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Token {
    Increment,
    Decrement,
    PointerIncrement,
    PointerDecrement,
    LoopStart,
    LoopEnd,
    Input,
    Output,
}

impl Token {
    pub fn parse(character: &char) -> Option<Self> {
        match character {
            '+' => Some(Self::Increment),
            '-' => Some(Self::Decrement),
            '>' => Some(Self::PointerIncrement),
            '<' => Some(Self::PointerDecrement),
            '[' => Some(Self::LoopStart),
            ']' => Some(Self::LoopEnd),
            ',' => Some(Self::Input),
            '.' => Some(Self::Output),
            _ => None,
        }
    }
}

pub struct JumpTable {
    jumps: HashMap<usize, usize>,
}

#[derive(Debug, PartialEq)]
pub enum JumpTableError {
    TooManyLoopStarts(usize),
    NoMatchingLoopEnd(usize),
}

impl JumpTable {
    pub fn from(tokens: &[Token]) -> Result<JumpTable, JumpTableError> {
        let mut jumps = HashMap::new();
        let mut start_loop_stack = Vec::new();

        for (position, token) in tokens.iter().enumerate() {
            match token {
                Token::LoopStart => start_loop_stack.push(position),
                Token::LoopEnd => {
                    let start = match start_loop_stack.pop() {
                        Some(x) => x,
                        None => return Err(JumpTableError::NoMatchingLoopEnd(position)),
                    };
                    jumps.insert(start, position);
                    jumps.insert(position, start);
                }
                _ => (),
            }
        }

        match start_loop_stack.len() {
            0 => Ok(Self { jumps }),
            _ => Err(JumpTableError::TooManyLoopStarts(start_loop_stack.len())),
        }
    }

    pub fn resolve(&self, position: &usize) -> Option<&usize> {
        self.jumps.get(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_token() {
        let characters = ['+', '-', '>', '<', '[', ']', ',', '.', ' ', 'k'];
        let tokens: Vec<_> = characters.iter().map(Token::parse).collect();

        let expected = vec![
            Some(Token::Increment),
            Some(Token::Decrement),
            Some(Token::PointerIncrement),
            Some(Token::PointerDecrement),
            Some(Token::LoopStart),
            Some(Token::LoopEnd),
            Some(Token::Input),
            Some(Token::Output),
            None,
            None,
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn create_jump_table_more_end_loops() {
        let tokens = [Token::LoopEnd];
        let table = JumpTable::from(&tokens);

        assert!(table.is_err());
        assert_eq!(table.err().unwrap(), JumpTableError::NoMatchingLoopEnd(0));
    }

    #[test]
    fn create_jump_table_more_start_loops() {
        let tokens = [Token::LoopStart];
        let table = JumpTable::from(&tokens);

        assert!(table.is_err());
        assert_eq!(table.err().unwrap(), JumpTableError::TooManyLoopStarts(1));
    }

    #[test]
    fn create_jump_table_nested_loop() {
        let tokens = [
            Token::LoopStart,
            Token::Increment,
            Token::LoopStart,
            Token::Increment,
            Token::LoopEnd,
            Token::Increment,
            Token::LoopEnd,
        ];
        let table = JumpTable::from(&tokens);

        assert!(table.is_ok());

        let table = table.unwrap();

        assert_eq!(table.resolve(&0), Some(&6));
        assert_eq!(table.resolve(&1), None);
        assert_eq!(table.resolve(&2), Some(&4));
        assert_eq!(table.resolve(&3), None);
        assert_eq!(table.resolve(&4), Some(&2));
        assert_eq!(table.resolve(&5), None);
        assert_eq!(table.resolve(&6), Some(&0));
    }
}
