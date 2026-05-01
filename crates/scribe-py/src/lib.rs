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

fn load_template(path: Option<&str>) -> PyResult<Option<Template>> {
    match path {
        None => Ok(None),
        Some(p) => Template::from_reference_doc(p)
            .map(Some)
            .map_err(|e| PyValueError::new_err(e.to_string())),
    }
}

/// Convert a Markdown string to ``.docx`` bytes.
///
/// If ``reference_doc`` is given it must point to a ``.docx`` file whose
/// styles will replace Folio's built-in ones.
#[pyfunction]
#[pyo3(signature = (markdown, reference_doc=None))]
fn convert<'py>(
    py: Python<'py>,
    markdown: &str,
    reference_doc: Option<&str>,
) -> PyResult<Bound<'py, PyBytes>> {
    let template = load_template(reference_doc)?;
    let bytes = py
        .allow_threads(|| scribe_core::convert_string_with_template(markdown, template.as_ref()))
        .map_err(map_err)?;
    Ok(PyBytes::new_bound(py, &bytes))
}

/// Convert a Markdown file at ``input`` into a ``.docx`` file at ``output``.
/// Relative image paths are resolved against the input file's parent directory.
#[pyfunction]
#[pyo3(signature = (input, output, reference_doc=None))]
fn convert_file(
    py: Python<'_>,
    input: &str,
    output: &str,
    reference_doc: Option<&str>,
) -> PyResult<()> {
    let template = load_template(reference_doc)?;
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

#[pymodule]
fn _folio(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(convert, m)?)?;
    m.add_function(wrap_pyfunction!(convert_file, m)?)?;
    m.add_function(wrap_pyfunction!(preview_html, m)?)?;
    m.add_function(wrap_pyfunction!(preview_standalone, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
