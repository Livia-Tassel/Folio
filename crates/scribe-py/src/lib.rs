//! Python bindings for Folio. Loaded by the `folio` Python package as
//! `folio._folio`. The wrapper layer only translates types and errors —
//! all conversion behavior lives in `scribe-core` and is tested there.

// PyO3 0.22's `#[pyfunction]` macro expansion includes an internal
// `PyErr::into()` round-trip that clippy flags as a useless conversion.
// Suppressing here keeps `cargo clippy -D warnings` clean without
// littering each wrapper with attributes.
#![allow(clippy::useless_conversion)]

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use scribe_core::{ConvertError, Template};

fn map_err(e: ConvertError) -> PyErr {
    match e {
        ConvertError::Read(io) => PyIOError::new_err(io.to_string()),
        ConvertError::Write(io) => PyIOError::new_err(io.to_string()),
        ConvertError::Emit(an) => PyValueError::new_err(an.to_string()),
        ConvertError::Template(te) => PyValueError::new_err(te.to_string()),
    }
}

fn pick_template(reference_doc: Option<&str>, theme: Option<&str>) -> PyResult<Option<Template>> {
    match (reference_doc, theme) {
        (None, None) => Ok(None),
        (Some(_), Some(_)) => Err(PyValueError::new_err(
            "reference_doc and theme are mutually exclusive",
        )),
        (Some(p), None) => Template::from_reference_doc(p)
            .map(Some)
            .map_err(|e| PyValueError::new_err(e.to_string())),
        (None, Some(name)) => Template::builtin(name).map(Some).map_err(|e| {
            let known = scribe_core::list_builtin_themes().join(", ");
            PyValueError::new_err(format!("{e} (known themes: {known})"))
        }),
    }
}

/// Convert a Markdown string to ``.docx`` bytes.
///
/// Pass ``reference_doc`` (path to a styled .docx) OR ``theme`` (a built-in
/// name from :func:`list_themes`) to override Folio's default styles. The
/// two are mutually exclusive.
#[pyfunction]
#[pyo3(signature = (markdown, reference_doc=None, theme=None))]
fn convert<'py>(
    py: Python<'py>,
    markdown: &str,
    reference_doc: Option<&str>,
    theme: Option<&str>,
) -> PyResult<Bound<'py, PyBytes>> {
    let template = pick_template(reference_doc, theme)?;
    let bytes = py
        .allow_threads(|| scribe_core::convert_string_with_template(markdown, template.as_ref()))
        .map_err(map_err)?;
    Ok(PyBytes::new_bound(py, &bytes))
}

/// Convert a Markdown file at ``input`` into a ``.docx`` file at ``output``.
/// Relative image paths are resolved against the input file's parent directory.
#[pyfunction]
#[pyo3(signature = (input, output, reference_doc=None, theme=None))]
fn convert_file(
    py: Python<'_>,
    input: &str,
    output: &str,
    reference_doc: Option<&str>,
    theme: Option<&str>,
) -> PyResult<()> {
    let template = pick_template(reference_doc, theme)?;
    py.allow_threads(|| {
        scribe_core::convert_file_with_template(input, output, template.as_ref())
    })
    .map_err(map_err)
}

/// Render a Markdown string as an HTML preview fragment (no ``<html>`` wrapper).
#[pyfunction]
fn preview_html(py: Python<'_>, markdown: &str) -> String {
    py.allow_threads(|| scribe_core::preview_html(markdown))
}

/// Render a Markdown string as a complete standalone HTML document.
#[pyfunction]
fn preview_standalone(py: Python<'_>, markdown: &str) -> String {
    py.allow_threads(|| scribe_core::preview_standalone(markdown))
}

/// Names of built-in themes accepted by ``convert(theme=...)``.
#[pyfunction]
fn list_themes() -> Vec<&'static str> {
    scribe_core::list_builtin_themes()
}

#[pymodule]
fn _folio(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(convert, m)?)?;
    m.add_function(wrap_pyfunction!(convert_file, m)?)?;
    m.add_function(wrap_pyfunction!(preview_html, m)?)?;
    m.add_function(wrap_pyfunction!(preview_standalone, m)?)?;
    m.add_function(wrap_pyfunction!(list_themes, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
