"""Type stubs for the folio package."""

__version__: str

def convert(
    markdown: str,
    reference_doc: str | None = None,
    theme: str | None = None,
) -> bytes:
    """Convert a Markdown string to .docx bytes.

    Args:
        markdown: The Markdown source text.
        reference_doc: Optional path to a reference .docx whose styles
            override Folio's built-in styles. Same semantics as
            Pandoc's --reference-doc.
        theme: Optional name of a built-in theme. See :func:`list_themes`.
            Mutually exclusive with ``reference_doc``.

    Raises:
        ValueError: if Markdown emission fails, both reference_doc and
            theme are given, the reference doc is invalid, or the theme
            name is unknown.
    """
    ...

def convert_file(
    input: str,
    output: str,
    reference_doc: str | None = None,
    theme: str | None = None,
) -> None:
    """Convert a Markdown file to a .docx file, resolving relative image
    paths against the input file's parent directory.

    Args:
        input: Path to the source .md file.
        output: Path to the destination .docx file.
        reference_doc: Optional path to a reference .docx whose styles
            override Folio's built-in styles.
        theme: Optional name of a built-in theme. See :func:`list_themes`.
            Mutually exclusive with ``reference_doc``.

    Raises:
        IOError: if the input cannot be read or the output cannot be written.
        ValueError: if Markdown emission fails, both reference_doc and
            theme are given, the reference doc is invalid, or the theme
            name is unknown.
    """
    ...

def preview_html(markdown: str) -> str:
    """Render Markdown as an HTML preview fragment (no <html> wrapper)."""
    ...

def preview_standalone(markdown: str) -> str:
    """Render Markdown as a complete standalone HTML document."""
    ...

def list_themes() -> list[str]:
    """Return the names of built-in themes accepted by ``theme=...``."""
    ...
