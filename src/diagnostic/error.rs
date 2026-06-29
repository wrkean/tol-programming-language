use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Diagnostic, Error, Debug)]
pub enum TolError {
    #[error("May nakita akong hindi ko kilalang karakter: `{character}`")]
    #[diagnostic(help("Baka hindi ito parte ng aking sintaks, subukan mo itong tanggalin"))]
    UnrecognizedCharacter {
        character: char,

        #[label("Hindi ko kilala ang karakter na ito")]
        span: SourceSpan,
    },

    #[error("May nakita akong hindi inaasahang token: `{token}`")]
    #[diagnostic(help("Subukan mong palitan ng `{expected}` ang `{token}`"))]
    UnexpectedToken {
        token: String,
        expected: String,

        #[label("Umasa ako na `{expected}` ang makikita ko, ngunit ito ang nakita ko")]
        span: SourceSpan,
    },

    #[error("May nakita akong invalid na pagsimula ng isang expresyon")]
    InvalidStartOfAnExpression {
        #[label("Hindi ito pwedeng magsimula ng isang expresyon")]
        span: SourceSpan,
    },
}
