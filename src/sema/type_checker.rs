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
            if let Err(diag) = self.type_check_statement(statement) {
                self.modul.add_diagnostic(diag);
            };
        }
    }

    fn type_check_statement(&mut self, statement: &Stmt) -> TolResult<()> {
        match statement.kind() {
            StmtKind::NameDeclaration {
                is_mutable,
                name,
                ty,
                rhs,
            } => self.type_check_name_declaration(statement),
            StmtKind::FunctionDeclaration {
                name,
                params,
                ret_ty,
                block,
            } => todo!(),
            StmtKind::Expression { expr } => todo!(),
        }
    }

    fn type_check_name_declaration(&mut self, statement: &Stmt) -> TolResult<()> {
        let StmtKind::NameDeclaration {
            is_mutable,
            name,
            ty,
            rhs,
        } = statement.kind()
        else {
            unreachable!()
        };

        let rhs_ty = self.infer_expression(rhs)?;

        match ty {
            Some(t) => {
                self.coerce(&rhs_ty, t).ok_or(TolDiagnostic::new_error(
                    TolError::InvalidAssignment {
                        lhs_ty_str: t.to_tol_str(),
                        rhs_ty_str: rhs_ty.to_tol_str(),
                        rhs_span: rhs.span().clone().into(),
                    },
                ))?;
            }
            None => {
                let id = statement.symbol_id().unwrap();
                let symbol = self.modul.get_symbol_mut(id).unwrap();
                symbol.set_type(rhs_ty);
            }
        }

        Ok(())
    }

    fn infer_expression(&mut self, expression: &Expr) -> TolResult<TolType> {
        match expression.kind() {
            ExprKind::Integer(_) => Ok(TolType::Numero),
            ExprKind::Float(token) => Ok(TolType::Lutang),
            ExprKind::Identifier(token) => self.infer_identifier(expression),
            ExprKind::Binary { .. } => self.infer_binary(expression),
            ExprKind::Block { statements } => todo!(),
        }
    }

    fn infer_identifier(&mut self, identifier: &Expr) -> TolResult<TolType> {
        let ExprKind::Identifier(token) = identifier.kind() else {
            unreachable!()
        };

        let id = identifier.symbol_id().unwrap();
        let symbol = self.modul.get_symbol(id).unwrap();
        match symbol.ty() {
            Some(ty) => Ok(ty),
            None => Err(TolDiagnostic::new_error(TolError::UnableToInferType {
                span: token.span().clone().into(),
            })),
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

        self.check_assignment(
            op,
            &lhs_type,
            &rhs_type,
            lhs.span().clone(),
            rhs.span().clone(),
        )?;

        Ok(TolType::Wala) // Assignment operation always evaluates to `wala` or void
    }

    fn infer_arithmetic(&mut self, binary: &Expr) -> TolResult<TolType> {
        let ExprKind::Binary { lhs, rhs, op } = binary.kind() else {
            unreachable!();
        };

        let lhs_type = self.infer_expression(lhs)?;
        let rhs_type = self.infer_expression(rhs)?;

        self.check_arithmetic_operator(
            op,
            &lhs_type,
            &rhs_type,
            lhs.span().clone(),
            rhs.span().clone(),
        )
    }

    fn check_assignment(
        &mut self,
        op: &Token,
        lhs_type: &TolType,
        rhs_type: &TolType,
        lhs_span: Span,
        rhs_span: Span,
    ) -> TolResult<()> {
        if lhs_type == rhs_type {
            return Ok(());
        }

        if self.coerce(rhs_type, lhs_type).is_some() {
            // Optionally insert an ImplicitCast node here.
            return Ok(());
        }

        Err(TolDiagnostic::new_error(TolError::InvalidOperandTypes {
            lhs_ty_str: lhs_type.to_tol_str(),
            rhs_ty_str: rhs_type.to_tol_str(),
            operator: op.lexeme().to_string(),
            lhs_span: lhs_span.into(),
            rhs_span: rhs_span.into(),
        }))
    }

    fn check_arithmetic_operator(
        &self,
        op: &Token,
        lhs_type: &TolType,
        rhs_type: &TolType,
        lhs_span: Span,
        rhs_span: Span,
    ) -> TolResult<TolType> {
        if lhs_type == rhs_type {
            return Ok(lhs_type.clone());
        }

        self.coerce(lhs_type, rhs_type)
            .ok_or(TolDiagnostic::new_error(TolError::InvalidOperandTypes {
                lhs_ty_str: lhs_type.to_tol_str(),
                rhs_ty_str: rhs_type.to_tol_str(),
                operator: op.lexeme().to_string(),
                lhs_span: lhs_span.into(),
                rhs_span: rhs_span.into(),
            }))
    }

    fn coerce(&self, from: &TolType, to: &TolType) -> Option<TolType> {
        if from == to {
            return Some(from.clone());
        }

        match (from, to) {
            (TolType::Numero, TolType::Lutang) | (TolType::Lutang, TolType::Numero) => {
                Some(TolType::Lutang)
            }
            _ => None,
        }
    }
}
