//! scribe-preview: AST → HTML approximation for live preview.
//!
//! The preview pane in the desktop app renders this HTML so the user
//! can see (roughly) what the `.docx` will look like, with ~1s latency
//! from keystroke to render.
//!
//! This renderer is intentionally simple and template-agnostic; the
//! template's style mapping affects fonts and spacing in the final
//! `.docx` but the preview always uses a sensible neutral baseline.

use scribe_ast::{Alignment, Block, Document, Inline};

/// Render a [`Document`] into an HTML fragment suitable for embedding
/// in a preview iframe. The returned string does NOT include `<html>` /
/// `<body>` tags — wrap it at the caller (Tauri or tests) with a shell
/// that loads preview.css.
pub fn render(doc: &Document) -> String {
    let mut out = String::with_capacity(4096);
    for block in &doc.blocks {
        render_block(block, &mut out);
    }
    for (label, blocks) in &doc.footnotes {
        out.push_str(&format!(
            r#"<div class="footnote" id="fn-{}"><span class="fn-label">[^{}]</span>"#,
            escape_html(label),
            escape_html(label)
        ));
        for b in blocks {
            render_block(b, &mut out);
        }
        out.push_str("</div>");
    }
    out
}

/// Render the full HTML document (with a minimal `<head>` and CSS link).
/// Useful for saving a preview snapshot or unit-testing the renderer.
pub fn render_standalone(doc: &Document) -> String {
    let body = render(doc);
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Folio Preview</title>
<style>
{css}
</style>
</head>
<body><main class="scribe-preview">{body}</main></body>
</html>"#,
        css = PREVIEW_CSS,
        body = body,
    )
}

/// Bundled CSS for the preview. Baseline academic-English look.
pub const PREVIEW_CSS: &str = r#"
body { font-family: "Times New Roman", Georgia, serif; font-size: 12pt; max-width: 8.5in; margin: 2em auto; line-height: 1.5; padding: 0 1.5em; color: #222; }
.scribe-preview h1 { font-size: 24pt; margin-top: 1.5em; }
.scribe-preview h2 { font-size: 18pt; margin-top: 1.25em; }
.scribe-preview h3 { font-size: 14pt; margin-top: 1em; }
.scribe-preview h4, .scribe-preview h5, .scribe-preview h6 { font-size: 12pt; margin-top: 0.75em; }
.scribe-preview code, .scribe-preview pre { font-family: Menlo, Consolas, monospace; font-size: 10.5pt; }
.scribe-preview pre { background: #f6f7f9; padding: 0.75em 1em; border-radius: 4px; overflow-x: auto; }
.scribe-preview code { background: #f0f1f3; padding: 0.1em 0.3em; border-radius: 3px; }
.scribe-preview blockquote { border-left: 3px solid #ccc; padding-left: 1em; color: #555; margin-left: 0; }
.scribe-preview table { border-collapse: collapse; margin: 1em 0; }
.scribe-preview th, .scribe-preview td { border: 1px solid #888; padding: 0.3em 0.6em; }
.scribe-preview th { background: #f0f1f3; }
.scribe-preview hr { border: none; border-top: 1px solid #ccc; margin: 2em 0; }
.scribe-preview .task { list-style: none; }
.scribe-preview .task input { margin-right: 0.5em; }
.scribe-preview .math-block { text-align: center; margin: 1em 0; font-family: "Latin Modern Math", "Cambria Math", serif; }
.scribe-preview .math-inline { font-family: "Latin Modern Math", "Cambria Math", serif; }
.scribe-preview img { max-width: 100%; height: auto; display: block; margin: 1em auto; }
.scribe-preview .footnote { font-size: 10pt; color: #555; border-top: 1px solid #eee; padding-top: 0.5em; margin-top: 1em; }
.scribe-preview .fn-label { color: #0563c1; font-weight: 600; margin-right: 0.4em; }
.scribe-preview a { color: #0563c1; }
"#;

fn render_block(block: &Block, out: &mut String) {
    match block {
        Block::Heading { level, content } => {
            let lvl = (*level).clamp(1, 6);
            out.push_str(&format!("<h{lvl}>"));
            render_inlines(content, out);
            out.push_str(&format!("</h{lvl}>"));
        }
        Block::Paragraph { content } => {
            out.push_str("<p>");
            render_inlines(content, out);
            out.push_str("</p>");
        }
        Block::BlockQuote { blocks } => {
            out.push_str("<blockquote>");
            for b in blocks {
                render_block(b, out);
            }
            out.push_str("</blockquote>");
        }
        Block::CodeBlock { lang, code } => {
            let class = if lang.is_empty() {
                String::new()
            } else {
                format!(" class=\"language-{}\"", escape_html(lang))
            };
            out.push_str(&format!("<pre><code{class}>"));
            out.push_str(&escape_html(code));
            out.push_str("</code></pre>");
        }
        Block::List {
            ordered,
            start,
            items,
        } => {
            let tag = if *ordered { "ol" } else { "ul" };
            let start_attr = if *ordered && *start > 1 {
                format!(" start=\"{start}\"")
            } else {
                String::new()
            };
            out.push_str(&format!("<{tag}{start_attr}>"));
            for item in items {
                if let Some(checked) = item.task {
                    let checked_attr = if checked { " checked" } else { "" };
                    out.push_str(&format!(
                        "<li class=\"task\"><input type=\"checkbox\" disabled{checked_attr}> "
                    ));
                } else {
                    out.push_str("<li>");
                }
                for b in &item.blocks {
                    match b {
                        // Loose lists wrap content in <p>; tight lists
                        // should render paragraphs as inline content only.
                        Block::Paragraph { content } if item.blocks.len() == 1 => {
                            render_inlines(content, out);
                        }
                        other => render_block(other, out),
                    }
                }
                out.push_str("</li>");
            }
            out.push_str(&format!("</{tag}>"));
        }
        Block::Table {
            alignments,
            header,
            rows,
        } => {
            out.push_str("<table>");
            if !header.is_empty() {
                out.push_str("<thead><tr>");
                for (i, cell) in header.iter().enumerate() {
                    let align = alignments.get(i).copied().unwrap_or(Alignment::None);
                    render_cell(cell, align, "th", out);
                }
                out.push_str("</tr></thead>");
            }
            out.push_str("<tbody>");
            for row in rows {
                out.push_str("<tr>");
                for (i, cell) in row.iter().enumerate() {
                    let align = alignments.get(i).copied().unwrap_or(Alignment::None);
                    render_cell(cell, align, "td", out);
                }
                out.push_str("</tr>");
            }
            out.push_str("</tbody></table>");
        }
        Block::ThematicBreak => out.push_str("<hr>"),
        Block::MathBlock { latex } => {
            out.push_str(r#"<div class="math-block">"#);
            out.push_str(&format!("$${}$$", escape_html(latex)));
            out.push_str("</div>");
        }
    }
}

fn render_cell(cell: &[Inline], align: Alignment, tag: &str, out: &mut String) {
    let style = match align {
        Alignment::None => "",
        Alignment::Left => " style=\"text-align:left\"",
        Alignment::Center => " style=\"text-align:center\"",
        Alignment::Right => " style=\"text-align:right\"",
    };
    out.push_str(&format!("<{tag}{style}>"));
    render_inlines(cell, out);
    out.push_str(&format!("</{tag}>"));
}

fn render_inlines(inlines: &[Inline], out: &mut String) {
    for inline in inlines {
        render_inline(inline, out);
    }
}

fn render_inline(inline: &Inline, out: &mut String) {
    match inline {
        Inline::Text(s) => out.push_str(&escape_html(s)),
        Inline::Strong(xs) => {
            out.push_str("<strong>");
            render_inlines(xs, out);
            out.push_str("</strong>");
        }
        Inline::Emphasis(xs) => {
            out.push_str("<em>");
            render_inlines(xs, out);
            out.push_str("</em>");
        }
        Inline::Strikethrough(xs) => {
            out.push_str("<del>");
            render_inlines(xs, out);
            out.push_str("</del>");
        }
        Inline::Code(s) => {
            out.push_str("<code>");
            out.push_str(&escape_html(s));
            out.push_str("</code>");
        }
        Inline::Link {
            url,
            title,
            content,
        } => {
            if is_safe_link_url(url) {
                out.push_str(&format!(
                    "<a href=\"{}\" title=\"{}\">",
                    escape_html(url),
                    escape_html(title)
                ));
                render_inlines(content, out);
                out.push_str("</a>");
            } else {
                render_inlines(content, out);
            }
        }
        Inline::Image { url, alt, title } => {
            out.push_str(&format!(
                "<img src=\"{}\" alt=\"{}\" title=\"{}\">",
                escape_html(url),
                escape_html(alt),
                escape_html(title)
            ));
        }
        Inline::FootnoteRef(label) => {
            out.push_str(&format!(
                r##"<sup class="fn-ref"><a href="#fn-{}">[{}]</a></sup>"##,
                escape_html(label),
                escape_html(label)
            ));
        }
        Inline::InlineMath(latex) => {
            out.push_str(r#"<span class="math-inline">"#);
            out.push_str(&format!("\\({}\\)", escape_html(latex)));
            out.push_str("</span>");
        }
        Inline::SoftBreak => out.push(' '),
        Inline::HardBreak => out.push_str("<br>"),
    }
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn is_safe_link_url(url: &str) -> bool {
    let trimmed = url.trim_start_matches(|c: char| c.is_ascii_whitespace() || c.is_control());
    let Some(colon) = trimmed.find(':') else {
        return true;
    };

    let first_path_char = ['/', '?', '#']
        .iter()
        .filter_map(|ch| trimmed.find(*ch))
        .min()
        .unwrap_or(usize::MAX);
    if colon > first_path_char {
        return true;
    }

    let scheme = &trimmed[..colon];
    if !is_url_scheme(scheme) {
        return false;
    }

    matches!(
        scheme.to_ascii_lowercase().as_str(),
        "http" | "https" | "mailto"
    )
}

fn is_url_scheme(scheme: &str) -> bool {
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_and_paragraph_round_trip() {
        let mut doc = Document::new();
        doc.push(Block::Heading {
            level: 1,
            content: vec![Inline::Text("Hi".into())],
        });
        doc.push(Block::Paragraph {
            content: vec![Inline::Text("Body".into())],
        });
        let html = render(&doc);
        assert_eq!(html, "<h1>Hi</h1><p>Body</p>");
    }

    #[test]
    fn escapes_html_in_text() {
        let mut doc = Document::new();
        doc.push(Block::Paragraph {
            content: vec![Inline::Text("<script>alert('x')</script>".into())],
        });
        let html = render(&doc);
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn bold_italic_strike() {
        let mut doc = Document::new();
        doc.push(Block::Paragraph {
            content: vec![
                Inline::Strong(vec![Inline::Text("b".into())]),
                Inline::Emphasis(vec![Inline::Text("i".into())]),
                Inline::Strikethrough(vec![Inline::Text("s".into())]),
            ],
        });
        let html = render(&doc);
        assert!(html.contains("<strong>b</strong>"));
        assert!(html.contains("<em>i</em>"));
        assert!(html.contains("<del>s</del>"));
    }

    #[test]
    fn ordered_list_with_start() {
        let mut doc = Document::new();
        doc.push(Block::List {
            ordered: true,
            start: 3,
            items: vec![scribe_ast::ListItem {
                task: None,
                blocks: vec![Block::Paragraph {
                    content: vec![Inline::Text("three".into())],
                }],
            }],
        });
        let html = render(&doc);
        assert!(html.contains("<ol start=\"3\">"));
        assert!(html.contains("<li>three</li>"));
    }

    #[test]
    fn task_list_renders_checkbox() {
        let mut doc = Document::new();
        doc.push(Block::List {
            ordered: false,
            start: 0,
            items: vec![scribe_ast::ListItem {
                task: Some(true),
                blocks: vec![Block::Paragraph {
                    content: vec![Inline::Text("done".into())],
                }],
            }],
        });
        let html = render(&doc);
        assert!(html.contains(r#"<input type="checkbox" disabled checked>"#));
    }

    #[test]
    fn math_inline_and_block() {
        let mut doc = Document::new();
        doc.push(Block::Paragraph {
            content: vec![Inline::InlineMath("E = mc^2".into())],
        });
        doc.push(Block::MathBlock {
            latex: "a + b = c".into(),
        });
        let html = render(&doc);
        assert!(html.contains(r#"<span class="math-inline">"#));
        assert!(html.contains(r#"<div class="math-block">"#));
    }

    #[test]
    fn standalone_includes_css() {
        let doc = Document::new();
        let html = render_standalone(&doc);
        assert!(html.contains("<!doctype html>"));
        assert!(html.contains(".scribe-preview"));
    }

    #[test]
    fn unsafe_link_scheme_renders_plain_content() {
        let mut doc = Document::new();
        doc.push(Block::Paragraph {
            content: vec![Inline::Link {
                url: "javascript:alert(1)".into(),
                title: String::new(),
                content: vec![Inline::Text("click".into())],
            }],
        });

        let html = render(&doc);
        assert_eq!(html, "<p>click</p>");
    }

    #[test]
    fn unsafe_link_scheme_with_control_chars_renders_plain_content() {
        let mut doc = Document::new();
        doc.push(Block::Paragraph {
            content: vec![Inline::Link {
                url: "java\nscript:alert(1)".into(),
                title: String::new(),
                content: vec![Inline::Text("click".into())],
            }],
        });

        let html = render(&doc);
        assert_eq!(html, "<p>click</p>");
    }

    #[test]
    fn safe_link_scheme_keeps_anchor() {
        let mut doc = Document::new();
        doc.push(Block::Paragraph {
            content: vec![Inline::Link {
                url: "https://example.com".into(),
                title: "site".into(),
                content: vec![Inline::Text("example".into())],
            }],
        });

        let html = render(&doc);
        assert!(html.contains(r#"<a href="https://example.com" title="site">example</a>"#));
    }
}
