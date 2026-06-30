use crate::{
    diagnostic::{TolDiagnostic, error::TolError},
    prelude::{Span, TolResult},
};

#[derive(Debug)]
pub enum TolType {
    Numero,
    Lutang,

    Wala,
}

impl TolType {
    pub fn from_str(value: &str, span: Span) -> TolResult<Self> {
        let ty = match value {
            "numero" => TolType::Numero,
            "lutang" => TolType::Lutang,
            _ => {
                return Err(TolDiagnostic::new_error(TolError::InvalidType {
                    invalid_type: value.to_string(),
                    type_span: span.into(),
                }));
            }
        };

        Ok(ty)
    }
}
