"""Tests for HTML-to-PDF conversion."""

from __future__ import annotations

import base64

import pytest

from converter import html_to_pdf_base64, html_to_pdf_bytes


def test_html_to_pdf_bytes_returns_valid_pdf() -> None:
    html = "<p>Hello, world!</p>"
    result = html_to_pdf_bytes(html)
    assert result[:5] == b"%PDF-"
    assert len(result) > 100


def test_html_to_pdf_base64_roundtrips() -> None:
    html = "<p><strong>Cover Letter</strong></p><p>Dear hiring manager...</p>"
    b64 = html_to_pdf_base64(html)
    decoded = base64.b64decode(b64)
    assert decoded[:5] == b"%PDF-"


def test_empty_html_raises() -> None:
    with pytest.raises(Exception):
        html_to_pdf_bytes("")
