use std::{env, fs};

use brainfudge::{
    lexer::{JumpTable, Token},
    runtime::State,
};

fn main() {
    let source_path = env::args()
        .nth(1)
        .expect("No path the source file was given");
    let source = fs::read_to_string(source_path).unwrap();

    let tokens = tokenize(&source);
    let jump_table = JumpTable::from(&tokens).unwrap();
    let mut state = State::new();

    while state.can_execute(&tokens) {
        state
            .execute_current_instruction(&tokens, &jump_table)
            .unwrap();
    }
}

fn tokenize(source: &str) -> Vec<Token> {
    source
        .chars()
        .map(|x| Token::parse(&x))
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .collect()
}
