use crate::{prelude::Span, token::Token};

pub struct Expr {
    kind: ExprKind,
    span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

pub enum ExprKind {
    Integer(Token),
    Float(Token),
    Identifier(Token),
    Binary {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: Token,
    },
}
