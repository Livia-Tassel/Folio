//! MathML → OMML transformer.
//!
//! MathML is the output of `latex2mathml`; OMML (Office MathML) is what
//! Microsoft Word embeds inside `<w:r>` runs for editable equations.
//! The two dialects have roughly parallel structure — `<mfrac>` in MathML
//! maps to `<m:f>` with `<m:num>`/`<m:den>` children in OMML, and so on.
//!
//! This transformer parses MathML into a small in-memory tree, then walks
//! the tree producing OMML. It handles the common constructs needed for
//! academic papers: fractions, radicals, sub/sup, under/over, matrices,
//! fences, and the core token elements. Exotic MathML is preserved as
//! best-effort text.

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;

use super::{MathError, Result};

/// Parse a MathML string and emit an OMML fragment.
///
/// The output is an `<m:oMath>` (for inline) or `<m:oMathPara>` element
/// (for block) wrapping converted child content. Namespace declarations
/// are included so the fragment can be dropped directly into a docx run.
pub fn transform(mathml: &str) -> Result<String> {
    let root = parse_mathml(mathml)?;
    let is_block = root
        .attrs
        .iter()
        .any(|(k, v)| k == "display" && v == "block");

    let mut buf = Cursor::new(Vec::new());
    let mut w = Writer::new(&mut buf);

    if is_block {
        let mut para = BytesStart::new("m:oMathPara");
        para.push_attribute((
            "xmlns:m",
            "http://schemas.openxmlformats.org/officeDocument/2006/math",
        ));
        w.write_event(Event::Start(para)).map_err(wr_err)?;
        let math = BytesStart::new("m:oMath");
        w.write_event(Event::Start(math)).map_err(wr_err)?;
        for child in &root.children {
            write_node(&mut w, child)?;
        }
        w.write_event(Event::End(BytesEnd::new("m:oMath")))
            .map_err(wr_err)?;
        w.write_event(Event::End(BytesEnd::new("m:oMathPara")))
            .map_err(wr_err)?;
    } else {
        let mut math = BytesStart::new("m:oMath");
        math.push_attribute((
            "xmlns:m",
            "http://schemas.openxmlformats.org/officeDocument/2006/math",
        ));
        w.write_event(Event::Start(math)).map_err(wr_err)?;
        for child in &root.children {
            write_node(&mut w, child)?;
        }
        w.write_event(Event::End(BytesEnd::new("m:oMath")))
            .map_err(wr_err)?;
    }

    let bytes = buf.into_inner();
    String::from_utf8(bytes).map_err(|e| MathError::MathMl(e.to_string()))
}

fn wr_err(e: impl std::fmt::Display) -> MathError {
    MathError::MathMl(e.to_string())
}

// ---------------------------------------------------------------------------
// Minimal MathML tree representation + parser.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Node {
    Element(Element),
    Text(String),
}

#[derive(Debug, Clone)]
struct Element {
    tag: String,
    attrs: Vec<(String, String)>,
    children: Vec<Node>,
}

fn parse_mathml(xml: &str) -> Result<Element> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    // Find root <math> element.
    let mut buf = Vec::new();
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if local_name(e.name().as_ref()) == b"math" {
                    return parse_element(&mut reader, e);
                } else {
                    return Err(MathError::MathMl(format!(
                        "expected <math>, got <{}>",
                        String::from_utf8_lossy(e.name().as_ref())
                    )));
                }
            }
            Ok(Event::Eof) => return Err(MathError::MathMl("empty mathml input".into())),
            Ok(_) => continue,
            Err(e) => return Err(MathError::MathMl(e.to_string())),
        }
    }
}

fn parse_element(reader: &mut Reader<&[u8]>, start: BytesStart) -> Result<Element> {
    let tag = String::from_utf8_lossy(local_name(start.name().as_ref())).into_owned();
    let attrs = start
        .attributes()
        .flatten()
        .map(|a| {
            let k = String::from_utf8_lossy(local_name(a.key.as_ref())).into_owned();
            let v = String::from_utf8_lossy(&a.value).into_owned();
            (k, v)
        })
        .collect();

    let mut children = Vec::new();
    let mut buf = Vec::new();
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let child = parse_element(reader, e)?;
                children.push(Node::Element(child));
            }
            Ok(Event::Empty(e)) => {
                let child_tag = String::from_utf8_lossy(local_name(e.name().as_ref())).into_owned();
                let child_attrs = e
                    .attributes()
                    .flatten()
                    .map(|a| {
                        let k = String::from_utf8_lossy(local_name(a.key.as_ref())).into_owned();
                        let v = String::from_utf8_lossy(&a.value).into_owned();
                        (k, v)
                    })
                    .collect();
                children.push(Node::Element(Element {
                    tag: child_tag,
                    attrs: child_attrs,
                    children: Vec::new(),
                }));
            }
            Ok(Event::Text(t)) => {
                let s = String::from_utf8_lossy(&t).into_owned();
                if !s.is_empty() {
                    children.push(Node::Text(s));
                }
            }
            Ok(Event::End(e)) => {
                if local_name(e.name().as_ref()) == local_name(start.name().as_ref()) {
                    return Ok(Element {
                        tag,
                        attrs,
                        children,
                    });
                } else {
                    return Err(MathError::MathMl(format!(
                        "end tag mismatch: </{}>",
                        String::from_utf8_lossy(e.name().as_ref())
                    )));
                }
            }
            Ok(Event::Eof) => {
                return Err(MathError::MathMl(format!("unexpected EOF inside <{tag}>",)));
            }
            Ok(_) => continue,
            Err(e) => return Err(MathError::MathMl(e.to_string())),
        }
    }
}

fn local_name(full: &[u8]) -> &[u8] {
    match full.iter().position(|b| *b == b':') {
        Some(i) => &full[i + 1..],
        None => full,
    }
}

// ---------------------------------------------------------------------------
// MathML → OMML walker.
// ---------------------------------------------------------------------------

fn write_node(w: &mut Writer<&mut Cursor<Vec<u8>>>, node: &Node) -> Result<()> {
    match node {
        Node::Element(e) => write_element(w, e),
        Node::Text(s) => write_plain_run(w, s, RunKind::Text),
    }
}

fn write_element(w: &mut Writer<&mut Cursor<Vec<u8>>>, el: &Element) -> Result<()> {
    match el.tag.as_str() {
        "mrow" => {
            for c in &el.children {
                write_node(w, c)?;
            }
            Ok(())
        }
        "mi" => write_element_as_run(w, el, RunKind::Identifier),
        "mn" => write_element_as_run(w, el, RunKind::Number),
        "mo" => write_element_as_run(w, el, RunKind::Operator),
        "mtext" => write_element_as_run(w, el, RunKind::Text),
        "mspace" => Ok(()),
        "mfrac" => write_mfrac(w, el),
        "msqrt" => write_msqrt(w, el),
        "mroot" => write_mroot(w, el),
        "msup" => write_sup(w, el, false),
        "msub" => write_sup(w, el, true),
        "msubsup" => write_msubsup(w, el),
        "munder" => write_limlow(w, el),
        "mover" => write_limupp_or_acc(w, el),
        "munderover" => write_munderover(w, el),
        "mtable" => write_mtable(w, el),
        "mfenced" => write_mfenced(w, el),
        // Unknown element — emit inner text to avoid losing content.
        _ => {
            for c in &el.children {
                write_node(w, c)?;
            }
            Ok(())
        }
    }
}

enum RunKind {
    Identifier,
    Number,
    Operator,
    Text,
}

fn write_element_as_run(
    w: &mut Writer<&mut Cursor<Vec<u8>>>,
    el: &Element,
    kind: RunKind,
) -> Result<()> {
    let text = collect_text(el);
    write_plain_run(w, &text, kind)
}

fn collect_text(el: &Element) -> String {
    let mut buf = String::new();
    for c in &el.children {
        match c {
            Node::Text(s) => buf.push_str(s),
            Node::Element(inner) => buf.push_str(&collect_text(inner)),
        }
    }
    buf
}

fn write_plain_run(w: &mut Writer<&mut Cursor<Vec<u8>>>, text: &str, kind: RunKind) -> Result<()> {
    // <m:r>
    //   <m:rPr>
    //     <m:sty m:val="p"/>  ← plain for operators/numbers/text; italic (default "i") for identifiers
    //   </m:rPr>
    //   <m:t>...</m:t>
    // </m:r>
    let style = match kind {
        RunKind::Identifier => "i", // italic (default MathML behaviour)
        RunKind::Number | RunKind::Operator | RunKind::Text => "p", // plain upright
    };

    w.write_event(Event::Start(BytesStart::new("m:r")))
        .map_err(wr_err)?;

    w.write_event(Event::Start(BytesStart::new("m:rPr")))
        .map_err(wr_err)?;
    let mut sty = BytesStart::new("m:sty");
    sty.push_attribute(("m:val", style));
    w.write_event(Event::Empty(sty)).map_err(wr_err)?;
    w.write_event(Event::End(BytesEnd::new("m:rPr")))
        .map_err(wr_err)?;

    w.write_event(Event::Start(BytesStart::new("m:t")))
        .map_err(wr_err)?;
    w.write_event(Event::Text(BytesText::new(text)))
        .map_err(wr_err)?;
    w.write_event(Event::End(BytesEnd::new("m:t")))
        .map_err(wr_err)?;

    w.write_event(Event::End(BytesEnd::new("m:r")))
        .map_err(wr_err)?;
    Ok(())
}

fn write_mfrac(w: &mut Writer<&mut Cursor<Vec<u8>>>, el: &Element) -> Result<()> {
    // MathML: <mfrac>num den</mfrac>
    // OMML:   <m:f><m:num>…</m:num><m:den>…</m:den></m:f>
    let (num, den) = first_two_children(el)?;
    w.write_event(Event::Start(BytesStart::new("m:f")))
        .map_err(wr_err)?;

    w.write_event(Event::Start(BytesStart::new("m:num")))
        .map_err(wr_err)?;
    write_node(w, num)?;
    w.write_event(Event::End(BytesEnd::new("m:num")))
        .map_err(wr_err)?;

    w.write_event(Event::Start(BytesStart::new("m:den")))
        .map_err(wr_err)?;
    write_node(w, den)?;
    w.write_event(Event::End(BytesEnd::new("m:den")))
        .map_err(wr_err)?;

    w.write_event(Event::End(BytesEnd::new("m:f")))
        .map_err(wr_err)?;
    Ok(())
}

fn write_msqrt(w: &mut Writer<&mut Cursor<Vec<u8>>>, el: &Element) -> Result<()> {
    // <m:rad><m:radPr><m:degHide m:val="1"/></m:radPr><m:deg/><m:e>…</m:e></m:rad>
    w.write_event(Event::Start(BytesStart::new("m:rad")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:radPr")))
        .map_err(wr_err)?;
    let mut hide = BytesStart::new("m:degHide");
    hide.push_attribute(("m:val", "1"));
    w.write_event(Event::Empty(hide)).map_err(wr_err)?;
    w.write_event(Event::End(BytesEnd::new("m:radPr")))
        .map_err(wr_err)?;
    w.write_event(Event::Empty(BytesStart::new("m:deg")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:e")))
        .map_err(wr_err)?;
    for c in &el.children {
        write_node(w, c)?;
    }
    w.write_event(Event::End(BytesEnd::new("m:e")))
        .map_err(wr_err)?;
    w.write_event(Event::End(BytesEnd::new("m:rad")))
        .map_err(wr_err)?;
    Ok(())
}

fn write_mroot(w: &mut Writer<&mut Cursor<Vec<u8>>>, el: &Element) -> Result<()> {
    // MathML: <mroot>radicand index</mroot>
    // OMML:   <m:rad><m:deg>…index…</m:deg><m:e>…radicand…</m:e></m:rad>
    let (radicand, index) = first_two_children(el)?;
    w.write_event(Event::Start(BytesStart::new("m:rad")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:deg")))
        .map_err(wr_err)?;
    write_node(w, index)?;
    w.write_event(Event::End(BytesEnd::new("m:deg")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:e")))
        .map_err(wr_err)?;
    write_node(w, radicand)?;
    w.write_event(Event::End(BytesEnd::new("m:e")))
        .map_err(wr_err)?;
    w.write_event(Event::End(BytesEnd::new("m:rad")))
        .map_err(wr_err)?;
    Ok(())
}

fn write_sup(w: &mut Writer<&mut Cursor<Vec<u8>>>, el: &Element, is_sub: bool) -> Result<()> {
    // msup: <m:sSup><m:e>base</m:e><m:sup>exp</m:sup></m:sSup>
    // msub: <m:sSub><m:e>base</m:e><m:sub>idx</m:sub></m:sSub>
    let (base, other) = first_two_children(el)?;
    let (outer, inner_tag) = if is_sub {
        ("m:sSub", "m:sub")
    } else {
        ("m:sSup", "m:sup")
    };
    w.write_event(Event::Start(BytesStart::new(outer)))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:e")))
        .map_err(wr_err)?;
    write_node(w, base)?;
    w.write_event(Event::End(BytesEnd::new("m:e")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new(inner_tag)))
        .map_err(wr_err)?;
    write_node(w, other)?;
    w.write_event(Event::End(BytesEnd::new(inner_tag)))
        .map_err(wr_err)?;
    w.write_event(Event::End(BytesEnd::new(outer)))
        .map_err(wr_err)?;
    Ok(())
}

fn write_msubsup(w: &mut Writer<&mut Cursor<Vec<u8>>>, el: &Element) -> Result<()> {
    // MathML: <msubsup>base sub sup</msubsup>
    // OMML:   <m:sSubSup><m:e>base</m:e><m:sub>sub</m:sub><m:sup>sup</m:sup></m:sSubSup>
    let elems: Vec<&Node> = child_elements_or_text(el);
    if elems.len() < 3 {
        return Err(MathError::MathMl("msubsup requires 3 children".into()));
    }
    w.write_event(Event::Start(BytesStart::new("m:sSubSup")))
        .map_err(wr_err)?;
    for (i, tag) in ["m:e", "m:sub", "m:sup"].iter().enumerate() {
        w.write_event(Event::Start(BytesStart::new(*tag)))
            .map_err(wr_err)?;
        write_node(w, elems[i])?;
        w.write_event(Event::End(BytesEnd::new(*tag)))
            .map_err(wr_err)?;
    }
    w.write_event(Event::End(BytesEnd::new("m:sSubSup")))
        .map_err(wr_err)?;
    Ok(())
}

fn write_limlow(w: &mut Writer<&mut Cursor<Vec<u8>>>, el: &Element) -> Result<()> {
    // munder → limLow
    let (base, lim) = first_two_children(el)?;
    w.write_event(Event::Start(BytesStart::new("m:limLow")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:e")))
        .map_err(wr_err)?;
    write_node(w, base)?;
    w.write_event(Event::End(BytesEnd::new("m:e")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:lim")))
        .map_err(wr_err)?;
    write_node(w, lim)?;
    w.write_event(Event::End(BytesEnd::new("m:lim")))
        .map_err(wr_err)?;
    w.write_event(Event::End(BytesEnd::new("m:limLow")))
        .map_err(wr_err)?;
    Ok(())
}

fn write_limupp_or_acc(w: &mut Writer<&mut Cursor<Vec<u8>>>, el: &Element) -> Result<()> {
    // mover: if "accent" attribute is true, render as <m:acc>; else <m:limUpp>.
    let is_accent = el
        .attrs
        .iter()
        .any(|(k, v)| k == "accent" && (v == "true" || v == "1"));
    let (base, overlay) = first_two_children(el)?;
    if is_accent {
        let ch = collect_text(match overlay {
            Node::Element(e) => e,
            _ => return write_limupp(w, base, overlay),
        });
        w.write_event(Event::Start(BytesStart::new("m:acc")))
            .map_err(wr_err)?;
        w.write_event(Event::Start(BytesStart::new("m:accPr")))
            .map_err(wr_err)?;
        let mut chr = BytesStart::new("m:chr");
        chr.push_attribute(("m:val", ch.as_str()));
        w.write_event(Event::Empty(chr)).map_err(wr_err)?;
        w.write_event(Event::End(BytesEnd::new("m:accPr")))
            .map_err(wr_err)?;
        w.write_event(Event::Start(BytesStart::new("m:e")))
            .map_err(wr_err)?;
        write_node(w, base)?;
        w.write_event(Event::End(BytesEnd::new("m:e")))
            .map_err(wr_err)?;
        w.write_event(Event::End(BytesEnd::new("m:acc")))
            .map_err(wr_err)?;
        Ok(())
    } else {
        write_limupp(w, base, overlay)
    }
}

fn write_limupp(w: &mut Writer<&mut Cursor<Vec<u8>>>, base: &Node, lim: &Node) -> Result<()> {
    w.write_event(Event::Start(BytesStart::new("m:limUpp")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:e")))
        .map_err(wr_err)?;
    write_node(w, base)?;
    w.write_event(Event::End(BytesEnd::new("m:e")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:lim")))
        .map_err(wr_err)?;
    write_node(w, lim)?;
    w.write_event(Event::End(BytesEnd::new("m:lim")))
        .map_err(wr_err)?;
    w.write_event(Event::End(BytesEnd::new("m:limUpp")))
        .map_err(wr_err)?;
    Ok(())
}

fn write_munderover(w: &mut Writer<&mut Cursor<Vec<u8>>>, el: &Element) -> Result<()> {
    // For generic munderover we map to n-ary structure with both bounds.
    // OMML: <m:nary><m:naryPr>…</m:naryPr><m:sub>under</m:sub><m:sup>over</m:sup><m:e>base</m:e></m:nary>
    let elems: Vec<&Node> = child_elements_or_text(el);
    if elems.len() < 3 {
        return Err(MathError::MathMl("munderover requires 3 children".into()));
    }
    let (base, under, over) = (elems[0], elems[1], elems[2]);
    let op_char = match base {
        Node::Element(e) if e.tag == "mo" => collect_text(e),
        _ => String::new(),
    };
    w.write_event(Event::Start(BytesStart::new("m:nary")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:naryPr")))
        .map_err(wr_err)?;
    if !op_char.is_empty() {
        let mut chr = BytesStart::new("m:chr");
        chr.push_attribute(("m:val", op_char.as_str()));
        w.write_event(Event::Empty(chr)).map_err(wr_err)?;
    }
    let mut limloc = BytesStart::new("m:limLoc");
    limloc.push_attribute(("m:val", "undOvr"));
    w.write_event(Event::Empty(limloc)).map_err(wr_err)?;
    w.write_event(Event::End(BytesEnd::new("m:naryPr")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:sub")))
        .map_err(wr_err)?;
    write_node(w, under)?;
    w.write_event(Event::End(BytesEnd::new("m:sub")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:sup")))
        .map_err(wr_err)?;
    write_node(w, over)?;
    w.write_event(Event::End(BytesEnd::new("m:sup")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:e")))
        .map_err(wr_err)?;
    w.write_event(Event::End(BytesEnd::new("m:e")))
        .map_err(wr_err)?;
    w.write_event(Event::End(BytesEnd::new("m:nary")))
        .map_err(wr_err)?;
    Ok(())
}

fn write_mtable(w: &mut Writer<&mut Cursor<Vec<u8>>>, el: &Element) -> Result<()> {
    // MathML: <mtable><mtr><mtd>…</mtd>…</mtr>…</mtable>
    // OMML:   <m:m><m:mPr>…</m:mPr><m:mr><m:e>…</m:e>…</m:mr>…</m:m>
    w.write_event(Event::Start(BytesStart::new("m:m")))
        .map_err(wr_err)?;
    // mPr could specify column-count; skipped for v1 (Word infers from rows).
    for row in &el.children {
        let Node::Element(row_el) = row else { continue };
        if row_el.tag != "mtr" && row_el.tag != "mlabeledtr" {
            continue;
        }
        w.write_event(Event::Start(BytesStart::new("m:mr")))
            .map_err(wr_err)?;
        for cell in &row_el.children {
            let Node::Element(cell_el) = cell else {
                continue;
            };
            if cell_el.tag != "mtd" {
                continue;
            }
            w.write_event(Event::Start(BytesStart::new("m:e")))
                .map_err(wr_err)?;
            for inner in &cell_el.children {
                write_node(w, inner)?;
            }
            w.write_event(Event::End(BytesEnd::new("m:e")))
                .map_err(wr_err)?;
        }
        w.write_event(Event::End(BytesEnd::new("m:mr")))
            .map_err(wr_err)?;
    }
    w.write_event(Event::End(BytesEnd::new("m:m")))
        .map_err(wr_err)?;
    Ok(())
}

fn write_mfenced(w: &mut Writer<&mut Cursor<Vec<u8>>>, el: &Element) -> Result<()> {
    // MathML: <mfenced open="(" close=")" separators=",">…</mfenced>
    // OMML:   <m:d><m:dPr><m:begChr m:val="("/><m:endChr m:val=")"/></m:dPr><m:e>…</m:e></m:d>
    let open = el
        .attrs
        .iter()
        .find(|(k, _)| k == "open")
        .map(|(_, v)| v.as_str())
        .unwrap_or("(");
    let close = el
        .attrs
        .iter()
        .find(|(k, _)| k == "close")
        .map(|(_, v)| v.as_str())
        .unwrap_or(")");
    w.write_event(Event::Start(BytesStart::new("m:d")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:dPr")))
        .map_err(wr_err)?;
    let mut beg = BytesStart::new("m:begChr");
    beg.push_attribute(("m:val", open));
    w.write_event(Event::Empty(beg)).map_err(wr_err)?;
    let mut end = BytesStart::new("m:endChr");
    end.push_attribute(("m:val", close));
    w.write_event(Event::Empty(end)).map_err(wr_err)?;
    w.write_event(Event::End(BytesEnd::new("m:dPr")))
        .map_err(wr_err)?;
    w.write_event(Event::Start(BytesStart::new("m:e")))
        .map_err(wr_err)?;
    for c in &el.children {
        write_node(w, c)?;
    }
    w.write_event(Event::End(BytesEnd::new("m:e")))
        .map_err(wr_err)?;
    w.write_event(Event::End(BytesEnd::new("m:d")))
        .map_err(wr_err)?;
    Ok(())
}

fn first_two_children(el: &Element) -> Result<(&Node, &Node)> {
    let mut iter = el.children.iter().filter(|c| !is_empty_text(c));
    let a = iter
        .next()
        .ok_or_else(|| MathError::MathMl(format!("<{}> missing first child", el.tag)))?;
    let b = iter
        .next()
        .ok_or_else(|| MathError::MathMl(format!("<{}> missing second child", el.tag)))?;
    Ok((a, b))
}

fn child_elements_or_text(el: &Element) -> Vec<&Node> {
    el.children.iter().filter(|c| !is_empty_text(c)).collect()
}

fn is_empty_text(n: &Node) -> bool {
    matches!(n, Node::Text(s) if s.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mathml(body: &str) -> String {
        format!("<math xmlns=\"http://www.w3.org/1998/Math/MathML\">{body}</math>")
    }

    fn block_mathml(body: &str) -> String {
        format!(
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">{body}</math>"
        )
    }

    #[test]
    fn simple_identifier() {
        let omml = transform(&mathml("<mi>x</mi>")).unwrap();
        assert!(omml.contains("<m:oMath"));
        assert!(omml.contains("<m:t>x</m:t>"));
    }

    #[test]
    fn fraction_emits_f_num_den() {
        let omml = transform(&mathml("<mfrac><mn>1</mn><mn>2</mn></mfrac>")).unwrap();
        assert!(omml.contains("<m:f>"), "missing m:f in {omml}");
        assert!(omml.contains("<m:num>"), "missing m:num");
        assert!(omml.contains("<m:den>"), "missing m:den");
        assert!(omml.contains("<m:t>1</m:t>"));
        assert!(omml.contains("<m:t>2</m:t>"));
    }

    #[test]
    fn sqrt_emits_rad_with_hidden_degree() {
        let omml = transform(&mathml("<msqrt><mi>x</mi></msqrt>")).unwrap();
        assert!(omml.contains("<m:rad>"));
        assert!(omml.contains("<m:degHide"));
        assert!(omml.contains("<m:e>"));
    }

    #[test]
    fn msup_emits_sSup() {
        let omml = transform(&mathml("<msup><mi>E</mi><mn>2</mn></msup>")).unwrap();
        assert!(omml.contains("<m:sSup>"));
        assert!(omml.contains("<m:sup>"));
    }

    #[test]
    fn msub_emits_sSub() {
        let omml = transform(&mathml("<msub><mi>x</mi><mn>0</mn></msub>")).unwrap();
        assert!(omml.contains("<m:sSub>"));
        assert!(omml.contains("<m:sub>"));
    }

    #[test]
    fn block_emits_oMathPara() {
        let omml = transform(&block_mathml("<mi>a</mi>")).unwrap();
        assert!(omml.contains("<m:oMathPara"), "got: {omml}");
        assert!(omml.contains("<m:oMath"));
    }

    #[test]
    fn matrix_emits_m_mr_e() {
        let omml = transform(&mathml(
            "<mtable><mtr><mtd><mi>a</mi></mtd><mtd><mi>b</mi></mtd></mtr></mtable>",
        ))
        .unwrap();
        assert!(omml.contains("<m:m>"));
        assert!(omml.contains("<m:mr>"));
        assert!(omml.contains("<m:e>"));
    }

    #[test]
    fn fenced_emits_d_with_chars() {
        let omml = transform(&mathml(
            "<mfenced open=\"[\" close=\"]\"><mi>x</mi></mfenced>",
        ))
        .unwrap();
        assert!(omml.contains("<m:d>"));
        assert!(omml.contains("[\""));
        assert!(omml.contains("]\""));
    }

    #[test]
    fn e_mc2_full_pipeline() {
        // Simulate what latex2mathml would produce for E = mc^2:
        let mml = r#"<math xmlns="http://www.w3.org/1998/Math/MathML"><mi>E</mi><mo>=</mo><mi>m</mi><msup><mi>c</mi><mn>2</mn></msup></math>"#;
        let omml = transform(mml).unwrap();
        assert!(omml.contains("<m:oMath"));
        assert!(omml.contains("<m:t>E</m:t>"));
        assert!(omml.contains("<m:t>=</m:t>"));
        assert!(omml.contains("<m:sSup>"));
    }

    #[test]
    fn identifier_is_italic_number_is_plain() {
        let omml = transform(&mathml("<mi>x</mi><mn>2</mn>")).unwrap();
        // identifier gets m:sty m:val="i", number gets "p"
        assert!(omml.contains(r#"m:val="i""#));
        assert!(omml.contains(r#"m:val="p""#));
    }
}
