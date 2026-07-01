use std::fmt;

use crate::{
    ast::{Param, expr::Expr},
    prelude::{Span, Spanned},
    token::Token,
    toltype::TolType,
};

#[derive(Debug)]
pub struct Stmt {
    kind: StmtKind,
    span: Span,
    symbol_id: Option<usize>,
}

impl Stmt {
    pub fn new_name_declaration(
        span: Span,
        is_mutable: bool,
        name: Token,
        ty: Option<TolType>,
        rhs: Expr,
    ) -> Self {
        Self {
            kind: StmtKind::NameDeclaration {
                is_mutable,
                name,
                ty,
                rhs,
            },
            span,
            symbol_id: None,
        }
    }

    pub fn new_expression(span: Span, expr: Expr) -> Self {
        Self {
            kind: StmtKind::Expression { expr },
            span,
            symbol_id: None,
        }
    }

    pub fn new_par(
        span: Span,
        name: Token,
        params: Spanned<Vec<Param>>,
        ret_ty: Option<TolType>,
        block: Expr,
    ) -> Self {
        Self {
            kind: StmtKind::FunctionDeclaration {
                name,
                params,
                ret_ty,
                block,
            },
            span,
            symbol_id: None,
        }
    }

    pub fn kind(&self) -> &StmtKind {
        &self.kind
    }

    pub fn kind_mut(&mut self) -> &mut StmtKind {
        &mut self.kind
    }

    pub fn set_symbol_id(&mut self, id: usize) {
        self.symbol_id = Some(id)
    }

    pub fn symbol_id(&self) -> Option<usize> {
        self.symbol_id
    }
}

#[derive(Debug)]
pub enum StmtKind {
    NameDeclaration {
        is_mutable: bool,
        name: Token,
        ty: Option<TolType>,
        rhs: Expr,
    },
    FunctionDeclaration {
        name: Token,
        params: Spanned<Vec<Param>>,
        ret_ty: Option<TolType>,
        block: Expr,
    },
    Expression {
        expr: Expr,
    },
}
