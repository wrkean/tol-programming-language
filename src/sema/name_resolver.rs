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
    had_error: bool,
}

impl<'sema> NameResolver<'sema> {
    pub fn new(analyzer_ctx: &'sema mut AnalyzerCtx, modul: &'sema mut Module) -> Self {
        Self {
            analyzer_ctx,
            modul,
            had_error: false,
        }
    }

    /// Runs the name resolver, returns true if an error occured during the time that it runs
    pub fn run(&mut self) -> bool {
        let mut ast = self.modul.take_ast(); // Temporary ownership
        for statement in ast.iter_mut() {
            if let Err(diag) = self.resolve_statement(statement) {
                self.modul.add_diagnostic(diag);
                self.had_error = true;
            };
        }

        self.modul.set_ast(ast);

        self.had_error
    }

    fn resolve_statement(&mut self, statement: &mut Stmt) -> TolResult<()> {
        match statement.kind_mut() {
            StmtKind::NameDeclaration { .. } => self.resolve_name_declaration(statement),
            StmtKind::FunctionDeclaration { .. } => self.resolve_par(statement),
            StmtKind::Expression { expr } => self.resolve_expression(expr),
        }
    }

    fn resolve_expression(&mut self, expression: &mut Expr) -> TolResult<()> {
        match expression.kind_mut() {
            ExprKind::Integer(token) => Ok(()),
            ExprKind::Float(token) => Ok(()),
            ExprKind::Identifier(token) => match self.analyzer_ctx.lookup_symbol(token.lexeme()) {
                Some(id) => {
                    expression.set_symbol_id(id);
                    Ok(())
                }
                None => Err(TolDiagnostic::new_error(TolError::UseOfUndeclaredName {
                    name: token.lexeme().to_string(),
                    span: token.span().clone().into(),
                })),
            },
            ExprKind::Binary { lhs, rhs, op } => {
                // NOTE: Does not propagate immediately, proceeds to rhs
                if let Err(diag) = self.resolve_expression(lhs) {
                    self.modul.add_diagnostic(diag);
                    self.had_error = true;
                }

                self.resolve_expression(rhs)
            }
            ExprKind::Block { statements } => {
                // NOTE: Does not propagate immediately
                for statement in statements {
                    if let Err(diag) = self.resolve_statement(statement) {
                        self.modul.add_diagnostic(diag);
                        self.had_error = true;
                    }
                }

                Ok(())
            }
        }
    }

    fn resolve_par(&mut self, par: &mut Stmt) -> TolResult<()> {
        let StmtKind::FunctionDeclaration {
            name,
            params,
            ret_ty,
            block,
        } = par.kind_mut()
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
        let id = self.declare_symbol(symbol)?;

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
                self.had_error = true;
            }
        }

        self.analyzer_ctx.enter_scope(); // Enter block scope
        self.resolve_expression(block)?;

        self.analyzer_ctx.exit_scope(); // Exit from block scope
        self.analyzer_ctx.exit_scope(); // Exit from params scope

        par.set_symbol_id(id);

        Ok(())
    }

    fn resolve_name_declaration(&mut self, name_declaration: &mut Stmt) -> TolResult<()> {
        let StmtKind::NameDeclaration {
            is_mutable,
            name,
            ty,
            rhs,
        } = name_declaration.kind_mut()
        else {
            unreachable!()
        };

        self.resolve_expression(rhs)?;

        let symbol = Symbol::new_name(
            name.lexeme().to_string(),
            name.span().clone(),
            *is_mutable,
            ty.clone(),
        );

        let id = self.declare_symbol(symbol)?;
        name_declaration.set_symbol_id(id);

        Ok(())
    }

    fn declare_symbol(&mut self, symbol: Symbol) -> TolResult<usize> {
        match self.analyzer_ctx.lookup_current_scope(symbol.name()) {
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
                let name = symbol.name().to_string();
                let id = self.modul.add_symbol(symbol);
                self.analyzer_ctx.add_symbol_id(name, id);

                Ok(id)
            }
        }
    }
}
