use clap::Parser;
use miette::SourceSpan;

use crate::{
    ast::{
        expr::{Expr, ExprKind},
        stmt::{Stmt, StmtKind},
    },
    diagnostic::{TolDiagnostic, error::TolError},
    module::Module,
    prelude::{Span, TolResult},
    sema::analyzer_ctx::AnalyzerCtx,
    symbol::SymbolKind,
    token::{Token, TokenKind},
    toltype::TolType,
};

pub struct TypeChecker<'sema> {
    modul: &'sema mut Module,
}

impl<'sema> TypeChecker<'sema> {
    pub fn new(modul: &'sema mut Module) -> Self {
        Self { modul }
    }

    pub fn run(&mut self) {
        let mut ast = self.modul.take_ast();

        for statement in ast.iter() {
            if let Err(diag) = self.check_statement(statement) {
                self.modul.add_diagnostic(diag);
            };
        }

        self.modul.set_ast(ast);
    }

    fn check_statement(&mut self, statement: &Stmt) -> TolResult<()> {
        match statement.kind() {
            StmtKind::NameDeclaration {
                is_mutable,
                name,
                ty,
                rhs,
            } => self.check_name_declaration(statement),
            StmtKind::FunctionDeclaration {
                name,
                params,
                ret_ty,
                block,
            } => self.check_par(statement),
            StmtKind::Expression { expr } => {
                self.infer_expression(expr)?;
                Ok(())
            }
        }
    }

    fn check_name_declaration(&mut self, name_declaration: &Stmt) -> TolResult<()> {
        let StmtKind::NameDeclaration {
            is_mutable,
            name,
            ty,
            rhs,
        } = name_declaration.kind()
        else {
            unreachable!()
        };

        let rhs_ty = self.infer_expression(rhs)?;

        match ty {
            Some(t) => self.check_assignable(t, &rhs_ty, rhs.span().clone())?,
            None => {
                let id = name_declaration.symbol_id().unwrap();
                let symbol = self.modul.get_symbol_mut(id).unwrap();
                symbol.set_type(rhs_ty);
            }
        }

        Ok(())
    }

    fn check_par(&mut self, par: &Stmt) -> TolResult<()> {
        let StmtKind::FunctionDeclaration {
            name,
            params,
            ret_ty,
            block,
        } = par.kind()
        else {
            unreachable!()
        };

        self.infer_expression(block)?;

        Ok(())
    }

    fn infer_expression(&mut self, expression: &Expr) -> TolResult<TolType> {
        match expression.kind() {
            ExprKind::Integer(_) => Ok(TolType::Numero),
            ExprKind::Float(token) => Ok(TolType::Lutang),
            ExprKind::Identifier(token) => self.infer_identifier(expression),
            ExprKind::Binary { .. } => self.infer_binary(expression),
            ExprKind::Block { statements } => {
                for statement in statements {
                    if let Err(diag) = self.check_statement(statement) {
                        self.modul.add_diagnostic(diag);
                    }
                }

                Ok(TolType::Wala) // WARN: Temporary
            }
        }
    }

    fn infer_identifier(&mut self, identifier: &Expr) -> TolResult<TolType> {
        let ExprKind::Identifier(token) = identifier.kind() else {
            unreachable!()
        };

        let id = identifier.symbol_id().unwrap();
        let symbol = self.modul.get_symbol(id).unwrap();
        match symbol.kind() {
            SymbolKind::Name { declared_type, .. } => match declared_type {
                Some(ty) => Ok(ty.clone()),
                None => Err(TolDiagnostic::new_error(TolError::UnableToInferType {
                    span: token.span().clone().into(),
                })),
            },

            SymbolKind::Function { .. } => {
                panic!("Hindi pa sinusuportahan ng tol ang first-class functions")
            }
        }
    }

    pub fn infer_binary(&mut self, binary: &Expr) -> TolResult<TolType> {
        let ExprKind::Binary { op, .. } = binary.kind() else {
            unreachable!();
        };

        match op.kind() {
            TokenKind::Equal => self.infer_assignment(binary),

            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash => {
                self.infer_arithmetic(binary)
            }

            _ => todo!("unsupported binary operator"),
        }
    }

    fn infer_assignment(&mut self, binary: &Expr) -> TolResult<TolType> {
        let ExprKind::Binary { lhs, rhs, op, .. } = binary.kind() else {
            unreachable!();
        };

        if !lhs.kind().is_lvalue() {
            return Err(TolDiagnostic::new_error(TolError::UnexpectedLValue {
                span: lhs.span().clone().into(),
            }));
        }

        let lhs_type = self.infer_expression(lhs)?;
        let rhs_type = self.infer_expression(rhs)?;

        self.check_assignable(&lhs_type, &rhs_type, rhs.span().clone())?;

        let left_symbol_id = lhs.symbol_id().unwrap();
        let left_symbol = self.modul.get_symbol_mut(left_symbol_id).unwrap();
        if left_symbol.ty().is_none() {
            left_symbol.set_type(rhs_type.clone());
        }

        Ok(TolType::Wala) // Assignment operation always evaluates to `wala` or void
    }

    fn infer_arithmetic(&mut self, binary: &Expr) -> TolResult<TolType> {
        let ExprKind::Binary { lhs, rhs, op } = binary.kind() else {
            unreachable!();
        };

        let lhs_type = self.infer_expression(lhs)?;
        let rhs_type = self.infer_expression(rhs)?;

        self.common_type(&lhs_type, &rhs_type)
            .ok_or(TolDiagnostic::new_error(TolError::InvalidOperandTypes {
                lhs_ty_str: lhs_type.to_tol_str(),
                rhs_ty_str: rhs_type.to_tol_str(),
                operator: op.lexeme().to_string(),
                lhs_span: lhs.span().clone().into(),
                rhs_span: rhs.span().clone().into(),
            }))
    }

    /// Can these two types be converted into one common type?
    fn common_type(&self, lhs: &TolType, rhs: &TolType) -> Option<TolType> {
        if lhs == rhs {
            return Some(lhs.clone());
        }

        match (lhs, rhs) {
            (TolType::Numero, TolType::Lutang) | (TolType::Lutang, TolType::Numero) => {
                Some(TolType::Lutang)
            }
            _ => None,
        }
    }

    fn check_assignable(&self, lhs: &TolType, rhs: &TolType, rhs_span: Span) -> TolResult<()> {
        if lhs == rhs {
            return Ok(());
        }

        match (lhs, rhs) {
            (TolType::Numero, TolType::Lutang) | (TolType::Lutang, TolType::Numero) => Ok(()),
            (TolType::DiAlam, _) => Ok(()),
            _ => Err(TolDiagnostic::new_error(TolError::InvalidAssignment {
                lhs_ty_str: lhs.to_tol_str(),
                rhs_ty_str: rhs.to_tol_str(),
                rhs_span: rhs_span.into(),
            })),
        }
    }
}
