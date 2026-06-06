"""Dual-mode entry point for the PDF conversion service (ADR-004).

- Lambda: Mangum wraps the FastAPI app.
- Local dev: uvicorn on :3004.
"""

from __future__ import annotations

from fastapi import FastAPI, HTTPException
from mangum import Mangum
from pydantic import BaseModel, Field

from converter import html_to_pdf_base64

app = FastAPI(
    title="deploy-baba-pdf",
    description="HTML-to-PDF conversion service for cover letters",
    version="0.1.0",
)


class PdfRequest(BaseModel):
    html: str = Field(..., min_length=10, description="HTML content to convert to PDF")


class PdfResponse(BaseModel):
    pdf_base64: str = Field(description="Base64-encoded PDF bytes")


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok", "service": "pdf"}


@app.post("/convert", response_model=PdfResponse)
async def convert(body: PdfRequest) -> PdfResponse:
    """Convert HTML to PDF and return base64-encoded bytes."""
    try:
        result = html_to_pdf_base64(body.html)
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"PDF conversion failed: {exc}") from exc
    return PdfResponse(pdf_base64=result)


handler = Mangum(app, lifespan="off")

if __name__ == "__main__":
    import uvicorn

    uvicorn.run("handler:app", host="0.0.0.0", port=3004, reload=True)
