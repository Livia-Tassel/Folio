//! Unified error type for Folio's conversion pipeline.

use std::io;

pub type Result<T> = std::result::Result<T, ConvertError>;

#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    #[error("failed to read input: {0}")]
    Read(#[source] io::Error),

    #[error("failed to write output: {0}")]
    Write(#[source] io::Error),

    #[error("docx emission failed: {0}")]
    Emit(#[source] anyhow::Error),

    #[error("reference template error: {0}")]
    Template(#[from] scribe_template::TemplateError),
}
