"""Type stubs for the folio package."""

__version__: str

def convert(markdown: str, reference_doc: str | None = None) -> bytes:
    """Convert a Markdown string to .docx bytes.

    Args:
        markdown: The Markdown source text.
        reference_doc: Optional path to a reference .docx whose styles
            override Folio's built-in styles. Same semantics as
            Pandoc's --reference-doc.

    Raises:
        ValueError: if Markdown emission fails or the reference doc
            is invalid.
    """
    ...

def convert_file(
    input: str,
    output: str,
    reference_doc: str | None = None,
) -> None:
    """Convert a Markdown file to a .docx file, resolving relative image
    paths against the input file's parent directory.

    Args:
        input: Path to the source .md file.
        output: Path to the destination .docx file.
        reference_doc: Optional path to a reference .docx whose styles
            override Folio's built-in styles.

    Raises:
        IOError: if the input cannot be read or the output cannot be written.
        ValueError: if Markdown emission fails or the reference doc
            is invalid.
    """
    ...

def preview_html(markdown: str) -> str:
    """Render Markdown as an HTML preview fragment (no <html> wrapper)."""
    ...

def preview_standalone(markdown: str) -> str:
    """Render Markdown as a complete standalone HTML document."""
    ...
