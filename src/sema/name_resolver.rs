use crate::{
    ast::{
        expr::{Expr, ExprKind},
        stmt::{Stmt, StmtKind},
    },
    diagnostic::{TolDiagnostic, error::TolError},
    module::Module,
    prelude::{Span, TolResult},
    sema::analyzer_ctx::AnalyzerCtx,
    symbol::Symbol,
    toltype::TolType,
};

pub struct NameResolver<'sema> {
    analyzer_ctx: &'sema mut AnalyzerCtx,
    modul: &'sema mut Module,
}

impl<'sema> NameResolver<'sema> {
    pub fn new(analyzer_ctx: &'sema mut AnalyzerCtx, modul: &'sema mut Module) -> Self {
        Self {
            analyzer_ctx,
            modul,
        }
    }

    pub fn run(&mut self) {
        let ast = self.modul.take_ast(); // Temporary ownership
        for statement in ast.iter() {
            if let Err(diag) = self.resolve_statement(statement) {
                self.modul.add_diagnostic(diag);
            };
        }

        self.modul.set_ast(ast);
    }

    fn resolve_statement(&mut self, statement: &Stmt) -> TolResult<()> {
        match statement.kind() {
            StmtKind::NameDeclaration {
                is_mutable,
                name,
                ty,
                rhs,
            } => self.resolve_name_declaration(statement),
            StmtKind::FunctionDeclaration {
                name,
                params,
                ret_ty,
                block,
            } => self.resolve_par(statement),
            StmtKind::Expression { expr } => self.resolve_expression(expr),
        }
    }

    fn resolve_expression(&mut self, expression: &Expr) -> TolResult<()> {
        match expression.kind() {
            ExprKind::Integer(token) => Ok(()),
            ExprKind::Float(token) => Ok(()),
            ExprKind::Identifier(token) => match self.analyzer_ctx.lookup_symbol(token.lexeme()) {
                Some(_) => Ok(()),
                None => Err(TolDiagnostic::new_error(TolError::UseOfUndeclaredName {
                    name: token.lexeme().to_string(),
                    span: token.span().clone().into(),
                })),
            },
            ExprKind::Binary { lhs, rhs, op } => {
                // NOTE: Does not propagate immediately, proceeds to rhs
                if let Err(diag) = self.resolve_expression(lhs) {
                    self.modul.add_diagnostic(diag);
                }

                self.resolve_expression(rhs)
            }
            ExprKind::Block { statements } => {
                // NOTE: Does not propagate immediately
                for statement in statements {
                    if let Err(diag) = self.resolve_statement(statement) {
                        self.modul.add_diagnostic(diag);
                    }
                }

                Ok(())
            }
        }
    }

    fn resolve_par(&mut self, par: &Stmt) -> TolResult<()> {
        let StmtKind::FunctionDeclaration {
            name,
            params,
            ret_ty,
            block,
        } = par.kind()
        else {
            unreachable!()
        };

        let symbol = Symbol::new_function(
            name.lexeme().to_string(),
            name.span().clone(),
            params.item().iter().map(|p| p.ty().clone()).collect(),
            params.span().clone(),
            ret_ty.clone().unwrap_or(TolType::Wala),
        );
        self.declare_symbol(symbol)?;

        self.analyzer_ctx.enter_scope(); // Enter params scope
        for param in params.item() {
            let symbol = Symbol::new_name(
                param.name().lexeme().to_string(),
                param.span().clone(),
                false,
                Some(param.ty().clone()),
            );

            // NOTE: Does not propagate immediately
            if let Err(diag) = self.declare_symbol(symbol) {
                self.modul.add_diagnostic(diag);
            }
        }

        self.analyzer_ctx.enter_scope(); // Enter block scope
        self.resolve_expression(block)?;

        self.analyzer_ctx.exit_scope(); // Exit from block scope
        self.analyzer_ctx.exit_scope(); // Exit from params scope

        Ok(())
    }

    fn resolve_name_declaration(&mut self, name_declaration: &Stmt) -> TolResult<()> {
        let StmtKind::NameDeclaration {
            is_mutable,
            name,
            ty,
            rhs,
        } = name_declaration.kind()
        else {
            unreachable!()
        };

        let symbol = Symbol::new_name(
            name.lexeme().to_string(),
            name.span().clone(),
            *is_mutable,
            ty.clone(),
        );

        self.declare_symbol(symbol)
    }

    fn declare_symbol(&mut self, symbol: Symbol) -> TolResult<()> {
        match self.analyzer_ctx.lookup_symbol(symbol.name()) {
            Some(id) => {
                let declared_symbol = self.modul.get_symbol(id).unwrap();
                let declared_span = declared_symbol.declared_span().clone().into();

                Err(TolDiagnostic::new_error(TolError::NameRedeclaration {
                    name: symbol.name().to_string(),
                    declared_span,
                    redeclared_span: symbol.declared_span().clone().into(),
                }))
            }
            None => {
                self.modul.add_symbol(symbol);

                Ok(())
            }
        }
    }
}
