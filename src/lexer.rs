use std::{iter::Peekable, str::Chars};

use crate::token::Token;

pub struct Lexer<'src> {
    source: &'src str,
    source_iter: Peekable<Chars<'src>>,
    start: usize,
    current: usize,
    tokens: Vec<Token<'src>>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            source_iter: source.chars().peekable(),
            start: 0,
            current: 0,
            tokens: Vec::new(),
        }
    }
}
