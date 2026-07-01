use std::collections::HashMap;

use crate::toltype::TolType;

pub struct AnalyzerCtx {
    symbol_scope: Vec<HashMap<String, usize>>,
    current_fn_return_type: TolType,
}

impl AnalyzerCtx {
    pub fn new() -> Self {
        Self {
            symbol_scope: vec![HashMap::new()],
            current_fn_return_type: TolType::DiAlam,
        }
    }

    pub fn enter_scope(&mut self) {
        self.symbol_scope.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.symbol_scope.pop();
    }

    pub fn lookup_symbol(&self, name: &str) -> Option<usize> {
        for scope in self.symbol_scope.iter().rev() {
            if let Some(&id) = scope.get(name) {
                return Some(id);
            }
        }

        None
    }

    pub fn lookup_current_scope(&self, name: &str) -> Option<usize> {
        self.symbol_scope.last().unwrap().get(name).cloned()
    }
}
