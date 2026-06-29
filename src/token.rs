use crate::prelude::Span;

#[derive(Clone)]
pub struct Token {
    lexeme: String,
    kind: TokenKind,
    span: Span,
}

impl Token {
    pub fn new(lexeme: String, kind: TokenKind, span: Span) -> Self {
        Self { lexeme, kind, span }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    IntLiteral,
    FloatLiteral,

    Eof,
}

impl TokenKind {
    pub fn infers_semicolon(&self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            RParen | RSquare | Identifier | IntLiteral | FloatLiteral
        )
    }

    pub fn precedence(&self) -> u8 {
        match self {
            TokenKind::Plus | TokenKind::Minus => 1,
            TokenKind::Star | TokenKind::Slash => 2,
            _ => 0,
        }
    }

    pub fn assoc(&self) -> Associativity {
        match self {
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash => {
                Associativity::Left
            }
            _ => Associativity::Right,
        }
    }
}

pub enum Associativity {
    Left,
    Right,
}
