# Tachyon Changelog

## [1.1.0] - 2026-02-16

### Added
- **Server Document Routes**: Full implementation of REST API endpoints
  - `POST /api/v1/documents` - Create new documents with validation
  - `GET /api/v1/documents/{id}` - Retrieve documents by ID
  - `PUT /api/v1/documents/{id}` - Update document content and metadata
  - `DELETE /api/v1/documents/{id}` - Delete documents (soft delete)
  - `GET /api/v1/documents` - List documents with pagination and filters
  - `GET /api/v1/documents/search` - Full-text search support
  - `POST /api/v1/render` - Markdown to HTML rendering

- **Application State**: Proper database integration
  - DatabasePool initialization with migrations
  - DocumentState with repository and renderer
  - Graceful shutdown handling

- **Structured Error Responses**: Enterprise-grade error handling
  - Error codes (VALIDATION_ERROR, INVALID_ID, NOT_FOUND, etc.)
  - Detailed error messages
  - HTTP status code mapping

- **Document Lifecycle**: Status and visibility management
  - Document statuses: draft, published, archived, deleted
  - Document visibility: public, private, restricted
  - Status transition validation

- **Web Frontend Build**: Tailwind CSS v4 integration
  - Added @tailwindcss/postcss plugin
  - Created postcss.config.js for CSS processing
  - Updated main.css to use new @import syntax

### Changed
- **TypeScript**: Fixed implicit `any` type in editor.ts theme event handler
- **Server Main**: Integrated DocumentState with proper initialization
- **Cargo.toml**: Added tachyon-database and tachyon-renderer dependencies
- **Vite Config**: Removed dynamic requires, using PostCSS config instead

### Fixed
- Tailwind CSS v4 build configuration
- TypeScript compilation errors (0 errors now)
- Document routes now return proper responses instead of NOT_IMPLEMENTED
- Database type compatibility in document listing

### Security
- Input validation for document creation and updates
- Path traversal prevention in file operations
- Tag name sanitization
- Title length validation (max 200 characters)

## [0.1.0] - 2024-02-16

### Added
- **Renderer Module**: Full implementation of markdown parsing with pulldown-cmark
  - Support for CommonMark and GitHub Flavored Markdown (GFM)
  - Multiple output formats (HTML, Plain Text, AST, Markdown pass-through)
  - Metadata extraction (title, word count, heading count, code blocks)
  - LRU cache for rendered documents with TTL support
  
- **Syntax Highlighting**: Tree-sitter based syntax highlighting
  - Support for 12 languages (Rust, Python, JavaScript, TypeScript, JSON, TOML, YAML, HTML, CSS, SQL, Bash, Markdown)
  - Three built-in themes (Light, Dark, High Contrast)
  - CSS stylesheet generation for themes
  - HTML output with span classes for styling

- **Web Frontend**: Modern TypeScript-based web interface
  - CodeMirror 6 markdown editor with syntax highlighting
  - Full-text search with debounced queries
  - Dark/Light theme toggle with system preference detection
  - Responsive navigation with mobile support
  - API client with authentication
  - Event bus for component communication

- **Desktop Authentication**: Enhanced authentication system
  - Server authentication with fallback to local-first mode
  - Local user sessions for offline operation
  - SHA-256 based deterministic user ID generation

- **Search Module**: Improved search functionality
  - Proper tag parsing from Tantivy documents
  - Date/time parsing for created_at fields with multiple format support
  - Async RBAC permission checking

### Changed
- **Edition Upgrade**: Updated desktop crates from Rust 2021 to 2024 edition
  - `tachyon-desktop`: 2021 → 2024
  - `tachyon-desktop-app`: 2021 → 2024
  
- **Error Handling**: Replaced `unwrap()`/`expect()` with proper error handling
  - Indexer field access now returns proper `SearchError` types
  - Cache capacity initialization uses safe fallback
  
- **Flake.nix**: Removed placeholder revision hashes for flexibility

### Fixed
- Markdown parser now fully functional instead of returning "Not implemented"
- Syntax highlighter now fully functional with tree-sitter integration
- Search API RBAC check now properly validates resources and sessions
- Query engine now properly parses tags and created_at from search results

### Security
- Desktop authentication uses proper server validation before falling back to local mode
- CSRF token support in HTMX requests
- XSS prevention with HTML escaping in search results
