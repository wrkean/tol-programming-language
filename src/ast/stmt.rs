use std::fmt;

use crate::{ast::expr::Expr, prelude::Span, token::Token, toltype::TolType};

pub struct Stmt {
    kind: StmtKind,
    span: Span,
}

impl Stmt {
    pub fn new_name_declaration(span: Span, name: Token, ty: Option<TolType>, rhs: Expr) -> Self {
        Self {
            kind: StmtKind::NameDeclaration { name, ty, rhs },
            span,
        }
    }

    pub fn new_expression(span: Span, expr: Expr) -> Self {
        Self {
            kind: StmtKind::Expression { expr },
            span,
        }
    }

    pub fn kind(&self) -> &StmtKind {
        &self.kind
    }
}

pub enum StmtKind {
    NameDeclaration {
        name: Token,
        ty: Option<TolType>,
        rhs: Expr,
    },
    Expression {
        expr: Expr,
    },
}
