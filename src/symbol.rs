use crate::{prelude::Span, toltype::TolType};

#[derive(Debug)]
pub struct Symbol {
    name: String,
    kind: SymbolKind,
    declared_span: Span,
}

impl Symbol {
    pub fn new_name(
        name: String,
        declared_span: Span,
        is_mutable: bool,
        declared_type: Option<TolType>,
    ) -> Self {
        Self {
            name,
            declared_span,
            kind: SymbolKind::Name {
                is_mutable,
                declared_type,
            },
        }
    }

    pub fn new_function(
        name: String,
        declared_span: Span,
        param_types: Vec<TolType>,
        param_span: Span,
        declared_return_type: TolType,
    ) -> Self {
        Self {
            name,
            declared_span,
            kind: SymbolKind::Function {
                param_types,
                param_span,
                declared_return_type,
            },
        }
    }

    /// Returns one of the following based on the symbol kind:
    /// - name declaration type
    /// - function return type
    /// - none
    pub fn ty(&self) -> Option<TolType> {
        match &self.kind {
            SymbolKind::Name { declared_type, .. } => declared_type.clone(),
            SymbolKind::Function {
                param_types,
                param_span,
                declared_return_type,
            } => Some(declared_return_type.clone()),
        }
    }

    pub fn set_type(&mut self, ty: TolType) {
        match &mut self.kind {
            SymbolKind::Name { declared_type, .. } => *declared_type = Some(ty),
            SymbolKind::Function {
                declared_return_type,
                ..
            } => *declared_return_type = ty,
        }
    }

    pub fn declared_span(&self) -> &Span {
        &self.declared_span
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub enum SymbolKind {
    /// Name declaration
    Name {
        is_mutable: bool,
        declared_type: Option<TolType>,
    },
    Function {
        param_types: Vec<TolType>,
        param_span: Span,
        declared_return_type: TolType,
    },
}
