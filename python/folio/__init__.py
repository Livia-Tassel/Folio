"""Folio — Markdown to polished .docx output, without the cleanup pass.

Public API:
    convert(markdown)               -> bytes        # .docx as bytes
    convert_file(input, output)     -> None         # md path -> docx path
    preview_html(markdown)          -> str          # html fragment
    preview_standalone(markdown)    -> str          # full <!doctype html> doc
"""

from __future__ import annotations

from ._folio import (
    __version__,
    convert,
    convert_file,
    preview_html,
    preview_standalone,
)

__all__ = [
    "__version__",
    "convert",
    "convert_file",
    "preview_html",
    "preview_standalone",
]
