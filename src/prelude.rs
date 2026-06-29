use std::ops::Range;

use crate::diagnostic::TolDiagnostic;

pub type Span = Range<usize>;
pub type TolResult<T> = Result<T, TolDiagnostic>;
