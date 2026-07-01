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

    #[error("May nakita akong hindi ko kilalang tipo: `{invalid_type}`")]
    InvalidType {
        invalid_type: String,

        #[label("Ito ay hindi isang valid na tipo sa tol")]
        type_span: SourceSpan,
    },

    #[error("May nakita akong pangalan na idineklara mo ulit sa kaparehong sakop: `{name}`")]
    #[diagnostic(help("Maaaring isa lamang na kaparehong pangalan ang nasa isang sakop"))]
    NameRedeclaration {
        name: String,

        #[label("Naideklara mo na ang `{name}` dito...")]
        declared_span: SourceSpan,

        #[label("...at nakita kong naideklara mo ulit dito")]
        redeclared_span: SourceSpan,
    },

    #[error("May nakita akong pangalan na ginamit ngunit hindi pa ito naideklara: `{name}`")]
    UseOfUndeclaredName {
        name: String,

        #[label("Hindi mo pa ito naideklara")]
        span: SourceSpan,
    },

    #[error(r#"Hindi ko magawang "i-infer" ang tipo"#)]
    UnableToInferType {
        #[label(r#"Hindi ko magawang "i-infer" ang tipo nito"#)]
        span: SourceSpan,
    },

    #[error(r#"Hindi ko magawang "i-infer" ang tipo dahil hindi pa ito na-ideklara"#)]
    UnableToInferTypeUndeclared {
        #[label(r#"Hindi ko magawang "i-infer" ang tipo dahil hindi pa ito na-ideklara"#)]
        span: SourceSpan,
    },

    #[error(
        "Invalid ang tipo ng mga operands: `{lhs_ty_str}` at `{rhs_ty_str}` gamit ang operasyong `{operator}`"
    )]
    InvalidOperandTypes {
        lhs_ty_str: String,
        rhs_ty_str: String,
        operator: String,

        #[label("Ito ay may tipong `{lhs_ty_str}`")]
        lhs_span: SourceSpan,

        #[label("Ito ay may tipong `{rhs_ty_str}`")]
        rhs_span: SourceSpan,
    },

    // NOTE: Add definition on what an lvalue is
    #[error("Umasa ako ng l-value ngunit iba ang nakita ko")]
    UnexpectedLValue {
        #[label("Hindi ito isang l-value")]
        span: SourceSpan,
    },

    #[error(r#"Hindi pwedeng "i-assign" ang `{rhs_ty_str}` sa `{lhs_ty_str}` "#)]
    InvalidAssignment {
        lhs_ty_str: String,
        rhs_ty_str: String,

        #[label(
            r#"Ito ay may tipong `{rhs_ty_str}`, hindi ito pwede "i-assign" sa `{lhs_ty_str}`"#
        )]
        rhs_span: SourceSpan,
    },
}
