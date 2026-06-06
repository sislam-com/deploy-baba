"""HTML-to-PDF conversion using WeasyPrint."""

from __future__ import annotations

import base64
from io import BytesIO

from weasyprint import HTML  # type: ignore[import-untyped]

COVER_LETTER_CSS = """\
@page {
    size: letter;
    margin: 1in 1.25in;
}
body {
    font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
    font-size: 11pt;
    line-height: 1.6;
    color: #1a1a1a;
}
p { margin: 0 0 0.8em 0; }
strong { font-weight: 600; }
em { font-style: italic; }
h1, h2, h3 { margin: 0.5em 0 0.3em 0; }
"""


def html_to_pdf_bytes(html_content: str) -> bytes:
    """Convert an HTML fragment to PDF bytes."""
    full_html = f"""\
<!DOCTYPE html>
<html><head>
<meta charset="utf-8">
<style>{COVER_LETTER_CSS}</style>
</head><body>{html_content}</body></html>"""

    buf = BytesIO()
    HTML(string=full_html).write_pdf(buf)
    return buf.getvalue()


def html_to_pdf_base64(html_content: str) -> str:
    """Convert an HTML fragment to a base64-encoded PDF string."""
    pdf_bytes = html_to_pdf_bytes(html_content)
    return base64.b64encode(pdf_bytes).decode("ascii")
