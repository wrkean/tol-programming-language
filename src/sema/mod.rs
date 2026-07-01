use crate::{
    ast::stmt::Stmt,
    module::Module,
    sema::{analyzer_ctx::AnalyzerCtx, name_resolver::NameResolver},
};

pub mod analyzer_ctx;
pub mod name_resolver;

pub struct SemanticAnalyzer<'m> {
    analyzer_ctx: AnalyzerCtx,
    modul: &'m mut Module,
}

impl<'m> SemanticAnalyzer<'m> {
    pub fn new(modul: &'m mut Module) -> Self {
        Self {
            analyzer_ctx: AnalyzerCtx::new(),
            modul,
        }
    }

    pub fn run(&mut self) {
        self.resolve_names();
    }

    fn resolve_names(&mut self) {
        let mut resolver = NameResolver::new(&mut self.analyzer_ctx, self.modul);
        resolver.run();
    }
}
