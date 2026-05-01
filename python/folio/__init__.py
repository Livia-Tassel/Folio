"""Folio — Markdown to polished .docx output, without the cleanup pass.

Public API:
    convert(markdown, reference_doc=None, theme=None) -> bytes
    convert_file(input, output, reference_doc=None, theme=None) -> None
    preview_html(markdown)          -> str          # html fragment
    preview_standalone(markdown)    -> str          # full <!doctype html> doc
    list_themes()                   -> list[str]    # built-in theme names
"""

from __future__ import annotations

from ._folio import (
    __version__,
    convert,
    convert_file,
    list_themes,
    preview_html,
    preview_standalone,
)

__all__ = [
    "__version__",
    "convert",
    "convert_file",
    "list_themes",
    "preview_html",
    "preview_standalone",
]
