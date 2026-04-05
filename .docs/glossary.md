# Tachyon Glossary

**Document ID:** TACHYON-GL-V1.0
**Date:** 2026-02-11
**Version:** 0.2.0-beta
**Status:** Released
**Accessibility:** WCAG 2.1 AA Compliant

---

## Table of Contents

1. [A-E](#a-e)
2. [F-L](#f-l)
3. [G-P](#g-p)
4. [M-R](#m-r)
5. [S-T](#s-t)
6. [U-Z](#u-z)

---

## A-E

### A

#### ADR (Architecture Decision Record)
**Definition:** Architecture Decision Record - A structured record of significant architectural decisions made throughout the project lifecycle.

**Usage:** Used to document design decisions, trade-offs, and rationale for architectural choices.

**Context:** Phase 1 (Architecture) through Phase 7 (Documentation).

### API (Application Programming Interface)

**Definition:** Application Programming Interface - A set of routines and protocols for building and interacting with software applications.

**Context:** Used by Tachyon Server Mode to provide programmatic access to Tachyon functionality.

### APM (Advanced Package Manager)

**Definition:** Advanced Package Manager - JavaScript package manager created by the Bun team.

**Context:** Used by Tachyon web interface for dependency management.

### ASCII (American Standard Code for Information Interchange)

**Definition:** ASCII - Character encoding standard for electronic communication.

**Context:** Used in Tachyon for text encoding and character set handling.

---

## F-L

### JIT (Just-In-Time Compilation)

**Definition:** Just-In-Time Compilation - Compiler that converts code to machine language at runtime rather than ahead-of-time.

**Context:** Tachyon uses JIT compilation for Markdown to HTML rendering.

### LRU (Least-Recently-Used Cache)

**Definition:** Least Recently Used Cache - Cache eviction policy that discards the least recently used items when the cache is full.

**Context:** Tachyon implements LRU cache for rendered HTML with role-based keys.

---

## G-P

### Git

**Definition:** Git - Distributed version control system.

**Context:** Tachyon uses Git for repository management, versioning, and collaboration.

### GUI (Graphical User Interface)

**Definition:** Graphical User Interface - User interface that uses graphical elements such as windows, menus, and icons.

**Context:** Tachyon Desktop Mode uses a native WebView for cross-platform GUI.

---

## M-R

### Markdown

**Definition:** Markdown - Lightweight markup language for creating formatted text.

**Context:** Tachyon uses CommonMark-compliant Markdown for document authoring.

---

## S-T

### SSG (Static Site Generator)

**Definition:** Static Site Generator - Tool that generates static HTML websites from source files.

**Context:** Tachyon provides Static Export Mode as an alternative to traditional SSGs.

---

## U-Z

### UTF-8 (Unicode Transformation Format)

**Definition:** UTF-8 - Character encoding capable of encoding all possible Unicode characters.

**Context:** Tachyon uses UTF-8 encoding for handling international characters and symbols.

---

## WSG (WebSocket)

**Definition:** WebSocket - WebSockets - Communication protocol providing full-duplex, bidirectional communication over a single TCP connection.

**Context:** Tachyon uses WebSocket for real-time updates and hot-reload synchronization.

---

## Tachyon-Specific Terms

### Tachyon

**Definition:** Tachyon - The knowledge management platform described in this documentation.

### BYOE (Bring Your Own Editor)

**Definition:** Bring Your Own Editor - Workflow pattern where users can use their preferred text editor while the system provides real-time preview.

### Frontmatter

**Definition:** Frontmatter - Metadata placed at the top of Markdown files in YAML or TOML format.

### RBAC (Role-Based Access Control)

**Definition:** Role-Based Access Control - Authorization mechanism that restricts access based on user roles and group memberships.

### BM25

**Definition:** BM25 - Best Matching 25 - Probabilistic relevance ranking algorithm used in full-text search.

### KaTeX

**Definition:** KaTeX - Fast web typesetting library for rendering LaTeX mathematical notation.

### Mermaid

**Definition:** Mermaid.js - JavaScript library for creating diagrams and visualizations from text descriptions.

### Tauri

**Definition:** Tauri - Framework for building lightweight, cross-platform desktop applications using web technologies.

### Axum

**Definition:** Axiom - The design system and visual identity of Tachyon.

### LWW (Last-Write-Wins)

**Definition:** Last-Write-Wins - Conflict resolution strategy where the most recent modification wins.

### SSE (Server-Sent Events)

**Definition:** Server-Sent Events - Kernel-level file system events for monitoring file system changes on Linux, macOS, and Windows.

### FSEvents (File System Events)

**Definition:** FSEvents - macOS-specific file system event API for high-performance file monitoring.

### SSD (Solid State Drive)

**Definition:** Solid State Drive - Data storage device using flash memory for persistent storage.

### SIMD (Single Instruction, Multiple Data)

**Definition:** SIMD - Single Instruction, Multiple Data - Parallel processing technique used for performance optimization.

---

## Performance Terms

### Latency

**Definition:** The time delay between initiating an action and its completion.

**Related Terms:**
- Hot-reload latency - Time from file save to visible update
- Rendering latency - Time from Markdown parse to HTML generation
- End-to-end latency - Total time from user action to complete response

### Throughput

**Definition:** The rate at which a system can process requests or data.

**Related Terms:**
- Request per second - Number of requests handled per second
- Concurrent users - Number of simultaneous users supported

### Cache Hit Rate

**Definition:** The percentage of requests served from cache without recomputation.

**Related Terms:**
- Cache miss - Request that is not in cache and requires recomputation
- Cache eviction - Removal of items from cache when capacity is reached

---

## Security Terms

### Authentication

**Definition:** The process of verifying the identity of a user or system.

**Related Terms:**
- Bearer token - Security token included in authorization header
- OAuth 2.0 - Authorization framework that allows third-party applications
- SAML - Security Assertion Markup Language for exchanging authentication data
- JWT (JSON Web Token) - JSON Web Token for authentication
- Kanidm - Identity and access management system

### Authorization

**Definition:** The process of determining what actions an authenticated user is permitted to perform.

### Access Control

**Definition:** See RBAC above.

### Content Redaction

**Definition:** The removal of sensitive content from documents before they are displayed to unauthorized users.

**Related Terms:**
- Internal blocks - Content marked with ::: internal directives
- Security through obscurity - Hiding sensitive content by returning 404 Not Found instead of 403 Forbidden

---

## Technical Terms

### CommonMark

**Definition:** CommonMark - Standardized Markdown specification for plain text formatting.

### SIMD

**Definition:** See SIMD above.

### TypeScript

**Definition:** TypeScript - Superset of JavaScript that adds static type definitions.

### HTTP/2

**Definition:** HTTP/2 - Hypertext Transfer Protocol version 2 for web communication.

### WebSocket

**Definition:** See WebSocket above.

### TOML

**Definition:** TOML - Tom's Obvious Minimal Language - Human-readable configuration file format.

### YAML

**Definition:** YAML - YAML Ain't Markup Language - Human-readable configuration file format.

---

## Platform-Specific Terms

### Windows

### IOCP (I/O Completion Ports)

**Definition:** IOCP - I/O Completion Ports on Windows - Communication ports for kernel-level file system events.

### ReadDirectoryChangesW

**Definition:** ReadDirectoryChangesW - Windows API for monitoring directory changes for file system efficiency.

---

## macOS

### FSEvents

**Definition:** FSEvents - See above.

---

## Linux

### inotify

**Definition:** inotify - Linux kernel subsystem for monitoring file system events.

---

## Network Terms

### HTTP/1

**Definition:** HTTP/1 - Hypertext Transfer Protocol version 1 for web communication.

### TCP (Transmission Control Protocol)

**Definition:** TCP - Transmission Control Protocol - Connection-oriented protocol for reliable network communication.

---

## Getting More Help

**Documentation:**
- [Online Documentation](https://docs.tachyon.org)
- [User Guide](./user_guide.md)
- [API Reference](./api_reference.md)
- [Installation Guide](./installation_guide.md)
- [Configuration Guide](./configuration_guide.md)
- [FAQ](./faq.md)

**Community:**
- [GitHub Issues](https://github.com/tachyon-org/tachyon/issues)
- [Discord Server](https://discord.gg/tachyon)
- [Matrix Room](https://matrix.to/#/tachyon:matrix.org)

**Professional Support:**
- [Enterprise Support](mailto:enterprise@tachyon.org)
- [Security Report](mailto:security@tachyon.org)

---

**Document Control**

| Version | Date | Author | Changes |
|---------|-------|--------|----------|
| 1.0 | 2026-02-11 | Brand Strategist | Initial glossary from verified implementation |

---

**Accessibility Statement:** This document is WCAG 2.1 AA compliant with proper heading structure, sufficient color contrast, and keyboard navigation support.
