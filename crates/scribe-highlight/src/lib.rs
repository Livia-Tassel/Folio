//! scribe-highlight: syntax highlighting for code blocks.
//!
//! Tokenizes source code with [`syntect`] and returns a sequence of
//! [`Token`]s (text + color), which the docx emitter turns into styled
//! character runs. Pure Rust, no external theme files — everything is
//! bundled via syntect's `default-fancy` feature.

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// A highlighted span of code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub text: String,
    /// Hex color WITHOUT the leading `#`, e.g. `"3A3A3A"`. `None` means
    /// use the default/body color (the docx emitter will leave the run
    /// uncolored and let the template decide).
    pub color: Option<String>,
    pub bold: bool,
    pub italic: bool,
}

/// Highlight `code` for the given language token (e.g. "rust").
///
/// If the language is unknown or empty, returns a single token with no
/// color — callers still get valid tokens, just unstyled.
pub fn highlight(code: &str, language: &str) -> Vec<Token> {
    static SYNTAXES: std::sync::OnceLock<SyntaxSet> = std::sync::OnceLock::new();
    static THEMES: std::sync::OnceLock<ThemeSet> = std::sync::OnceLock::new();

    let syntaxes = SYNTAXES.get_or_init(SyntaxSet::load_defaults_newlines);
    let themes = THEMES.get_or_init(ThemeSet::load_defaults);

    let theme = themes.themes.get("InspiredGitHub").unwrap_or_else(|| {
        themes
            .themes
            .values()
            .next()
            .expect("syntect ships at least one theme")
    });

    let syntax =
        find_syntax(syntaxes, language).unwrap_or_else(|| syntaxes.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut out = Vec::new();

    for line in LinesWithEndings::from(code) {
        match highlighter.highlight_line(line, syntaxes) {
            Ok(spans) => {
                for (style, text) in spans {
                    out.push(to_token(style, text.to_string()));
                }
            }
            Err(_) => {
                // Syntax error during highlighting — fall back to plain text.
                out.push(Token {
                    text: line.to_string(),
                    color: None,
                    bold: false,
                    italic: false,
                });
            }
        }
    }
    out
}

fn find_syntax<'a>(
    syntaxes: &'a SyntaxSet,
    language: &str,
) -> Option<&'a syntect::parsing::SyntaxReference> {
    if language.is_empty() {
        return None;
    }
    // Try direct name match first, then extension, then token.
    syntaxes
        .find_syntax_by_token(language)
        .or_else(|| syntaxes.find_syntax_by_extension(language))
        .or_else(|| syntaxes.find_syntax_by_name(language))
}

fn to_token(style: Style, text: String) -> Token {
    let color = {
        let c = style.foreground;
        // InspiredGitHub's default body color is near-black #323232;
        // treat very dark colors as "default" (None) so the template's
        // code-block style wins.
        if c.r < 0x40 && c.g < 0x40 && c.b < 0x40 {
            None
        } else {
            Some(format!("{:02X}{:02X}{:02X}", c.r, c.g, c.b))
        }
    };
    let bold = style
        .font_style
        .contains(syntect::highlighting::FontStyle::BOLD);
    let italic = style
        .font_style
        .contains(syntect::highlighting::FontStyle::ITALIC);
    Token {
        text,
        color,
        bold,
        italic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_rust_returns_multiple_tokens() {
        let tokens = highlight("fn main() { let x = 1; }", "rust");
        assert!(tokens.len() > 1, "expected multiple tokens, got {tokens:?}");
        let joined: String = tokens.iter().map(|t| t.text.as_str()).collect();
        assert!(joined.contains("fn"));
        assert!(joined.contains("main"));
    }

    #[test]
    fn unknown_language_falls_back_to_plain_text() {
        let tokens = highlight("some text", "notalanguage");
        let joined: String = tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(joined.trim_end(), "some text");
    }

    #[test]
    fn empty_language_returns_plain_tokens() {
        let tokens = highlight("just text", "");
        let joined: String = tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(joined.trim_end(), "just text");
    }

    #[test]
    fn empty_code_returns_no_tokens() {
        let tokens = highlight("", "rust");
        assert!(tokens.is_empty());
    }
}
