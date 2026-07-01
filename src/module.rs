use std::{mem, sync::Arc};

use crate::{
    ast::{pretty_printer::ASTPrettyPrinter, stmt::Stmt},
    diagnostic::TolDiagnostic,
    symbol::Symbol,
    token::Token,
};

pub struct Module {
    source_code: Arc<str>,
    diagnostics: Option<Vec<TolDiagnostic>>,
    ast: Vec<Stmt>,
    symbol_table: Vec<Symbol>,
}

impl Module {
    pub fn new(source_code: impl Into<Arc<str>>) -> Self {
        Self {
            source_code: source_code.into(),
            diagnostics: None,
            ast: Vec::new(),
            symbol_table: Vec::new(),
        }
    }

    pub fn set_diagnostics(&mut self, diagnostics: Vec<TolDiagnostic>) {
        self.diagnostics = Some(diagnostics)
    }

    pub fn add_diagnostic(&mut self, diagnostic: TolDiagnostic) {
        self.diagnostics
            .as_mut()
            .expect("diagnostics not set, this is a compiler bug")
            .push(diagnostic);
    }

    pub fn report_diagnostics(&mut self) {
        let diagnostics = mem::take(
            self.diagnostics
                .as_mut()
                .expect("diagnostics not set, this is a compiler bug"),
        );

        for diag in diagnostics.into_iter() {
            eprintln!(
                "{:?}",
                miette::Report::with_source_code(
                    miette::Report::new(diag),
                    Arc::clone(&self.source_code)
                )
            )
        }
    }

    pub fn add_statement(&mut self, statement: Stmt) {
        self.ast.push(statement);
    }

    pub fn display_ast(&self) {
        let mut pretty_printer = ASTPrettyPrinter::new(&self.ast, 4);
        pretty_printer.run();
    }

    pub fn source_code(&self) -> &str {
        &self.source_code
    }

    pub fn source_code_arc(&self) -> Arc<str> {
        Arc::clone(&self.source_code)
    }

    pub fn add_symbol(&mut self, symbol: Symbol) -> usize {
        self.symbol_table.push(symbol);

        self.symbol_table.len() - 1
    }

    pub fn get_symbol(&mut self, id: usize) -> Option<&Symbol> {
        self.symbol_table.get(id)
    }

    pub fn ast(&self) -> &[Stmt] {
        &self.ast
    }

    pub fn take_ast(&mut self) -> Vec<Stmt> {
        mem::take(&mut self.ast)
    }

    pub fn set_ast(&mut self, ast: Vec<Stmt>) {
        self.ast = ast;
    }
}
