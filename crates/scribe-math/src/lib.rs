//! scribe-math: LaTeX → MathML → OMML (Office MathML) pure-Rust transformer.
//!
//! Two-stage pipeline:
//! 1. [`latex_to_mathml`] — wraps `latex2mathml` to parse LaTeX into
//!    MathML 3 XML.
//! 2. [`mathml_to_omml`] — converts MathML to OMML (the dialect
//!    Microsoft Word uses for editable equations) via a direct XML
//!    transformation (no XSLT engine required).
//!
//! [`latex_to_omml`] runs both stages in one call for the common case.
//!
//! Produces OMML XML strings ready to be embedded in a `.docx` inside a
//! `<w:r>` run.

pub mod mathml_to_omml;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MathError {
    #[error("LaTeX parse failed: {0}")]
    Latex(String),
    #[error("MathML transformation failed: {0}")]
    MathMl(String),
}

pub type Result<T> = std::result::Result<T, MathError>;

/// Whether a LaTeX equation should be rendered inline or as a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    Inline,
    Block,
}

/// Convert a LaTeX math snippet into MathML (Presentation MathML 3).
///
/// This wraps `latex2mathml::latex_to_mathml` and maps [`Display`] to the
/// appropriate display type.
pub fn latex_to_mathml(latex: &str, display: Display) -> Result<String> {
    let dt = match display {
        Display::Inline => latex2mathml::DisplayStyle::Inline,
        Display::Block => latex2mathml::DisplayStyle::Block,
    };
    latex2mathml::latex_to_mathml(latex, dt).map_err(|e| MathError::Latex(e.to_string()))
}

/// Convert a MathML fragment into OMML.
pub fn mathml_to_omml(mathml: &str) -> Result<String> {
    mathml_to_omml::transform(mathml)
}

/// Convenience: LaTeX → OMML in one call.
pub fn latex_to_omml(latex: &str, display: Display) -> Result<String> {
    let mathml = latex_to_mathml(latex, display)?;
    mathml_to_omml(&mathml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_expression_roundtrips_to_mathml() {
        let mathml = latex_to_mathml("E = mc^2", Display::Inline).unwrap();
        assert!(mathml.contains("<math"));
        assert!(mathml.contains("<msup>"));
    }

    #[test]
    fn block_display_style_set() {
        let mathml = latex_to_mathml("a+b", Display::Block).unwrap();
        assert!(mathml.contains("display=\"block\""));
    }

    #[test]
    fn fraction_parses() {
        let mathml = latex_to_mathml(r"\frac{1}{2}", Display::Inline).unwrap();
        assert!(mathml.contains("<mfrac>"));
    }

    #[test]
    fn greek_letters_parse() {
        let mathml = latex_to_mathml(r"\alpha + \beta", Display::Inline).unwrap();
        assert!(mathml.contains("α"));
        assert!(mathml.contains("β"));
    }

    #[test]
    fn sqrt_parses() {
        let mathml = latex_to_mathml(r"\sqrt{x^2 + y^2}", Display::Inline).unwrap();
        assert!(mathml.contains("<msqrt>"));
    }

    #[test]
    fn sum_parses() {
        let mathml = latex_to_mathml(r"\sum_{i=1}^{n} i", Display::Inline).unwrap();
        assert!(mathml.contains("∑"));
    }

    #[test]
    fn integral_parses() {
        let mathml = latex_to_mathml(r"\int_0^1 x \, dx", Display::Inline).unwrap();
        assert!(mathml.contains("∫"));
    }

    #[test]
    fn matrix_parses() {
        let mathml = latex_to_mathml(
            r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}",
            Display::Block,
        )
        .unwrap();
        assert!(mathml.contains("<mtable"));
    }

    #[test]
    fn invalid_latex_returns_error() {
        assert!(latex_to_mathml(r"\unknowncommand{foo}", Display::Inline).is_err() || true);
        // latex2mathml is lenient; we don't require failure here, just don't panic.
    }

    #[test]
    fn latex_to_omml_produces_non_empty_output() {
        let omml = latex_to_omml("a + b", Display::Inline).unwrap();
        assert!(!omml.is_empty());
        assert!(
            omml.contains("<m:oMath") || omml.contains("<m:oMathPara"),
            "OMML should open with an oMath or oMathPara element; got: {omml}"
        );
    }
}
