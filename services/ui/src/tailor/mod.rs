//! Resume tailoring pipeline — `POST /api/admin/tailor`.
//!
//! Orchestrates: parse JD keywords → match bullets → generate grounded rewrites → render DOCX/PDF.
//!
//! Module layout:
//! - `matcher`   — pure-Rust token-overlap scorer (no LLM, no secrets)
//! - `parser`    — LLM call: extract keyword list + skill categories from JD     (W-RST.4.4)
//! - `generator` — LLM call: grounded rewrite of matched bullets                 (W-RST.4.5)
//! - `renderer`  — DOCX/PDF rendering, reuses xtask path                         (W-RST.4.6)

#[allow(dead_code)] // route handler lands in W-RST.4.7
pub mod matcher;
