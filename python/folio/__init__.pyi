"""Type stubs for the folio package."""

__version__: str

def convert(markdown: str) -> bytes:
    """Convert a Markdown string to .docx bytes.

    Raises:
        ValueError: if Markdown emission fails.
    """
    ...

def convert_file(input: str, output: str) -> None:
    """Convert a Markdown file to a .docx file, resolving relative image
    paths against the input file's parent directory.

    Raises:
        IOError: if the input cannot be read or the output cannot be written.
        ValueError: if Markdown emission fails.
    """
    ...

def preview_html(markdown: str) -> str:
    """Render Markdown as an HTML preview fragment (no <html> wrapper)."""
    ...

def preview_standalone(markdown: str) -> str:
    """Render Markdown as a complete standalone HTML document."""
    ...
