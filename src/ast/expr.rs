use std::fmt;

use crate::{ast::stmt::Stmt, prelude::Span, token::Token};

#[derive(Debug)]
pub struct Expr {
    kind: ExprKind,
    span: Span,
    symbol_id: Option<usize>,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self {
            kind,
            span,
            symbol_id: None,
        }
    }

    pub fn new_block(span: Span, statements: Vec<Stmt>) -> Self {
        Self {
            kind: ExprKind::Block { statements },
            span,
            symbol_id: None,
        }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn kind(&self) -> &ExprKind {
        &self.kind
    }

    pub fn kind_mut(&mut self) -> &mut ExprKind {
        &mut self.kind
    }

    pub fn set_symbol_id(&mut self, id: usize) {
        self.symbol_id = Some(id);
    }

    pub fn symbol_id(&self) -> Option<usize> {
        self.symbol_id
    }
}

#[derive(Debug)]
pub enum ExprKind {
    Integer(Token),
    Float(Token),
    Identifier(Token),
    Binary {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: Token,
    },
    Block {
        statements: Vec<Stmt>,
    },
}

impl ExprKind {
    pub fn is_lvalue(&self) -> bool {
        use ExprKind::*;
        matches!(self, Identifier(_))
    }
}

// ============= USED FOR TESTS ONLY =============
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprKind::Integer(tok) | ExprKind::Float(tok) | ExprKind::Identifier(tok) => {
                write!(f, "{}", tok.lexeme())
            }
            ExprKind::Binary { lhs, rhs, op } => {
                write!(f, "({} {} {})", lhs, op.lexeme(), rhs)
            }
            _ => unimplemented!(),
        }
    }
}
