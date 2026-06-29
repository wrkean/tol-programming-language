use std::{mem, sync::Arc};

use crate::{diagnostic::TolDiagnostic, token::Token};

pub struct Module {
    source_code: Arc<str>,
    diagnostics: Option<Vec<TolDiagnostic>>,
    // ast: Vec<Stmt>,
}

impl Module {
    pub fn new(source_code: impl Into<Arc<str>>) -> Self {
        Self {
            source_code: source_code.into(),
            diagnostics: None,
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

    pub fn source_code(&self) -> &str {
        &self.source_code
    }

    pub fn source_code_arc(&self) -> Arc<str> {
        Arc::clone(&self.source_code)
    }
}
