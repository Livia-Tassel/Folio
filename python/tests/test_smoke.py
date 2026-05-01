"""End-to-end smoke tests for the Folio Python bindings.

These tests exercise the FFI boundary: bytes flow correctly, strings
round-trip, errors map to Python exceptions, file IO works against
real paths. Behavior of the conversion itself is covered by Rust
unit tests in scribe-core / scribe-docx.
"""

from __future__ import annotations

import os
from pathlib import Path

import pytest

import folio


def test_module_exposes_version() -> None:
    assert isinstance(folio.__version__, str)
    assert folio.__version__.count(".") >= 2


def test_convert_returns_zip_bytes() -> None:
    result = folio.convert("# Hello\n\nWorld.")
    assert isinstance(result, bytes)
    # .docx is a ZIP — magic bytes are b"PK".
    assert result[:2] == b"PK"
    # A minimal docx is several KB, never empty.
    assert len(result) > 1024


def test_convert_handles_unicode_and_math() -> None:
    md = "# 你好\n\nThe formula is $E = mc^2$.\n"
    result = folio.convert(md)
    assert result[:2] == b"PK"


def test_convert_file_writes_output(tmp_path: Path) -> None:
    src = tmp_path / "in.md"
    dst = tmp_path / "out.docx"
    src.write_text("# Title\n\nBody paragraph.\n", encoding="utf-8")
    folio.convert_file(str(src), str(dst))
    assert dst.exists()
    assert dst.stat().st_size > 1024
    assert dst.read_bytes()[:2] == b"PK"


def test_convert_file_missing_input_raises_ioerror(tmp_path: Path) -> None:
    nope = tmp_path / "does-not-exist.md"
    with pytest.raises(IOError):
        folio.convert_file(str(nope), str(tmp_path / "out.docx"))


def test_preview_html_returns_html_fragment() -> None:
    html = folio.preview_html("# Heading\n\nbody")
    assert isinstance(html, str)
    assert "<h1" in html.lower()
    assert "heading" in html.lower()


def test_preview_standalone_includes_doctype() -> None:
    html = folio.preview_standalone("# Title")
    assert "<!doctype html" in html.lower() or "<!DOCTYPE html" in html


def test_convert_releases_gil_for_concurrent_calls() -> None:
    # Smoke test for `py.allow_threads` — two threads convert simultaneously
    # and both should produce valid output. If the GIL were not released this
    # would still pass functionally; the goal here is to exercise the path
    # and confirm thread-safety doesn't blow up.
    import threading

    results: list[bytes] = []
    errors: list[BaseException] = []

    def worker() -> None:
        try:
            results.append(folio.convert("# concurrent\n\ntest"))
        except BaseException as e:  # noqa: BLE001
            errors.append(e)

    threads = [threading.Thread(target=worker) for _ in range(4)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    assert not errors, f"thread errors: {errors}"
    assert len(results) == 4
    assert all(r[:2] == b"PK" for r in results)
