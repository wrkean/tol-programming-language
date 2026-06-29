use crate::prelude::Span;

pub struct Token<'src> {
    lexeme: &'src str,
    kind: TokenKind,
    span: Span,
}

impl<'src> Token<'src> {
    pub fn new(lexeme: &'src str, kind: TokenKind, span: Span) -> Self {
        Self { lexeme, kind, span }
    }
}

pub enum TokenKind {
    Par,
    Kung,
    Kungdi,
    Kungwala,
    Ibalik,

    LParen,
    RParen,
    LSquare, // [
    RSquare, // ]
    LBrace,
    RBrace,
    Colon,
    Semicolon,
    Lesser,
    LesserEq,
    Greater,
    GreaterEq,
    Equal,
    EqualEq, // ==
    Plus,
    Minus,
    Star,
    Slash,

    Identifier,

    Eof,
}
