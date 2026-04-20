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

fn normalize_latex(latex: &str) -> String {
    let normalized = latex
        .replace(r"\mathcal{", r"\mathscr{")
        .replace(r"\tfrac", r"\frac")
        .replace(r"\dots", r"\ldots");
    let normalized = rewrite_text_commands(&normalized);
    let normalized = rewrite_cases_environments(&normalized);
    rewrite_top_level_newlines(&normalized)
}

fn rewrite_cases_environments(latex: &str) -> String {
    let begin = r"\begin{cases}";
    let end = r"\end{cases}";
    let mut out = String::with_capacity(latex.len());
    let mut rest = latex;

    while let Some(start) = rest.find(begin) {
        out.push_str(&rest[..start]);
        let after_begin = &rest[start + begin.len()..];
        let Some(stop) = after_begin.find(end) else {
            out.push_str(&rest[start..]);
            return out;
        };

        let body = after_begin[..stop].trim();
        out.push_str(r"\left\{\begin{matrix}");
        out.push_str(body);
        out.push_str(r"\end{matrix}\right.");
        rest = &after_begin[stop + end.len()..];
    }

    out.push_str(rest);
    out
}

fn rewrite_text_commands(latex: &str) -> String {
    rewrite_braced_command(latex, r"\text{", |body| {
        let escaped = body.replace(' ', r"\ ");
        format!(r"\mathrm{{{escaped}}}")
    })
}

fn rewrite_braced_command<F>(latex: &str, command: &str, mut rewriter: F) -> String
where
    F: FnMut(&str) -> String,
{
    let mut out = String::with_capacity(latex.len());
    let mut rest = latex;

    while let Some(start) = rest.find(command) {
        out.push_str(&rest[..start]);
        let body_start = start + command.len();
        let mut depth = 1usize;
        let mut end_idx = None;

        for (offset, ch) in rest[body_start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end_idx = Some(body_start + offset);
                        break;
                    }
                }
                _ => {}
            }
        }

        let Some(end) = end_idx else {
            out.push_str(&rest[start..]);
            return out;
        };

        let body = &rest[body_start..end];
        out.push_str(&rewriter(body));
        rest = &rest[end + 1..];
    }

    out.push_str(rest);
    out
}

fn rewrite_top_level_newlines(latex: &str) -> String {
    if latex.contains(r"\begin{") || !latex.contains(r"\\") {
        return latex.to_string();
    }

    format!(r"\begin{{matrix}} {} \end{{matrix}}", latex.trim())
}

fn extract_parse_error(mathml: &str) -> Option<String> {
    let marker = "[PARSE ERROR:";
    let start = mathml.find(marker)?;
    let tail = &mathml[start + marker.len()..];
    let end = tail.find(']')?;
    Some(tail[..end].trim().to_string())
}

/// Convert a LaTeX math snippet into MathML (Presentation MathML 3).
///
/// This wraps `latex2mathml::latex_to_mathml` and maps [`Display`] to the
/// appropriate display type.
pub fn latex_to_mathml(latex: &str, display: Display) -> Result<String> {
    let normalized = normalize_latex(latex);
    let dt = match display {
        Display::Inline => latex2mathml::DisplayStyle::Inline,
        Display::Block => latex2mathml::DisplayStyle::Block,
    };
    let mathml = latex2mathml::latex_to_mathml(&normalized, dt)
        .map_err(|e| MathError::Latex(e.to_string()))?;
    if let Some(parse_error) = extract_parse_error(&mathml) {
        return Err(MathError::Latex(parse_error));
    }
    Ok(mathml)
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
    fn invalid_latex_does_not_panic() {
        // latex2mathml is lenient — we don't require Err here, just that
        // the call returns without panicking. Both Ok and Err are fine.
        let _ = latex_to_mathml(r"\unknowncommand{foo}", Display::Inline);
    }

    #[test]
    fn mathcal_is_normalized_to_supported_command() {
        let mathml = latex_to_mathml(r"\mathcal{C}_0", Display::Inline).unwrap();
        assert!(!mathml.contains("PARSE ERROR"));
        assert!(mathml.contains('C'));
    }

    #[test]
    fn tfrac_is_normalized_to_frac() {
        let mathml = latex_to_mathml(r"\tfrac{4}{3}", Display::Inline).unwrap();
        assert!(mathml.contains("<mfrac>"));
    }

    #[test]
    fn cases_is_rewritten_to_matrix_form() {
        let mathml = latex_to_mathml(
            r"S_{ij} = \begin{cases} X_{ij} & \text{if} \\ 0 & \text{else} \end{cases}",
            Display::Block,
        )
        .unwrap();
        assert!(!mathml.contains("PARSE ERROR"));
        assert!(mathml.contains("<mtable"));
    }

    #[test]
    fn text_command_is_rewritten_to_supported_style() {
        let mathml = latex_to_mathml(r"X \in \text{FP16}^{n \times d}", Display::Inline).unwrap();
        assert!(!mathml.contains("PARSE ERROR"));
        assert!(mathml.contains("<math"));
    }

    #[test]
    fn top_level_newlines_are_wrapped_for_stacked_equations() {
        let mathml = latex_to_mathml(r"S(C_1)=0.75 \\ S(C_2)=0.25", Display::Block).unwrap();
        assert!(mathml.contains("<mtable"));
    }

    #[test]
    fn dots_is_normalized_to_supported_command() {
        let mathml = latex_to_mathml(r"[t_1,\dots,t_7]", Display::Inline).unwrap();
        assert!(!mathml.contains("PARSE ERROR"));
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
