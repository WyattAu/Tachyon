# ADR-112: SSG Documentation Site

## Status
Accepted

## Context
Tachyon needs a documentation site that can be built from markdown source files and deployed to GitHub Pages.

## Decision
Use the custom `tachyon-ssg` crate to build the documentation site. The site is configured via `documentation/site.toml` and deployed to GitHub Pages via `.github/workflows/docs.yml`.

## Consequences
- Documentation is built as part of CI pipeline
- Site features: sidebar navigation, TOC, breadcrumbs, prev/next, KaTeX math, admonitions, syntax highlighting
- No external SSG dependency (Docusaurus, MkDocs, etc.)
- Deployed at https://wyattau.github.io/Tachyon

## Alternatives Considered
- Docusaurus: Heavy Node.js dependency, not Rust-native
- MkDocs Material: Python dependency, limited customization
- Hugo: Go dependency, template system learning curve
