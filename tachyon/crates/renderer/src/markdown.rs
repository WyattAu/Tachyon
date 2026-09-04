//! Markdown rendering.
//!
//! Thin re-export of the published [`docs_pipeline`] crate — the single
//! implementation of the markdown pipeline (GFM, wikilinks, admonitions,
//! embeds, block references, TOC extraction, sanitization).

pub use docs_pipeline::markdown::*;
