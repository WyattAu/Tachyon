# Plugin SDK Architecture Design (G.1)

## 1. Overview

The Tachyon plugin system uses a WASM-based runtime (the `plugin-runtime` crate) to execute third-party code in a sandboxed environment. This document covers the remaining work needed to ship a complete plugin ecosystem:

- SDK documentation and client libraries
- Template repository for new plugins
- Plugin marketplace for discovery and distribution
- Hardening of the existing sandbox model

## 2. Current State

The `plugin-runtime` crate provides the foundation:

- **Runtime engine**: Uses `wasmtime` for WASM compilation and execution.
- **Host interface**: `HostFunctions` trait exposes controlled operations (document CRUD, search, metadata queries) to guest plugins.
- **Plugin registry**: In-memory registry stores plugin metadata (name, version, declared permissions, entrypoint).
- **Execution model**: Plugins are compiled to `.wasm` files, loaded at startup, and invoked via the host function dispatch layer. All execution is confined to the WASM sandbox.

What exists today is sufficient for internal/bundled plugins. It lacks the developer experience tooling and governance infrastructure needed for third-party contributions.

## 3. SDK Design

### 3.1 Language Support

Two official SDKs:

- **Rust SDK** (`tachyon-plugin-sdk` crate): Wraps `wit-bindgen` generated types, provides a `Plugin` trait with default implementations.
- **TypeScript SDK** (`@tachyon/plugin-sdk` npm package): Targets `wasm-bindgen`/`jco` for WASM output. Provides typed wrappers over host functions.

Both SDKs consume the same WIT interface definition (`plugin.wit`), ensuring behavioral parity.

### 3.2 Plugin Trait / Interface

```rust
pub trait Plugin {
    fn init(ctx: &PluginContext) -> Result<()>;
    fn execute(ctx: &PluginContext, input: &[u8]) -> Result<Vec<u8>>;
    fn shutdown(ctx: &PluginContext) -> Result<()>;
}
```

TypeScript equivalent mirrors this with `init`, `execute`, and `shutdown` methods on an exported object.

### 3.3 Lifecycle

1. **Load**: Runtime reads the `.wasm` binary, validates the WIT interface, and checks declared permissions.
2. **Init**: Calls `init` with a `PluginContext` containing configuration and capability grants.
3. **Execute**: Calls `execute` on each invocation. Plugins are stateless between calls unless they use the provided key-value store host function.
4. **Shutdown**: Calls `shutdown` on graceful termination. Allows cleanup of resources.

### 3.4 API Scopes

Plugins declare required permissions in their manifest (`plugin.toml`):

| Scope                | Description                                      |
|----------------------|--------------------------------------------------|
| `read_documents`     | Read document content and metadata               |
| `write_documents`    | Create, update, delete documents                 |
| `manage_spaces`      | Create, reconfigure, delete spaces               |
| `admin`              | User management, system configuration, indexing  |

The runtime enforces scope boundaries at the host function layer. Undeclared scopes are denied regardless of caller identity.

### 3.5 Version Pinning

Plugins pin a minimum Tachyon API version in their manifest:

```toml
[plugin]
name = "my-plugin"
version = "1.2.0"
min_tachyon_api = "0.3.0"
```

The runtime rejects plugins whose `min_tachyon_api` exceeds the host's API version. This prevents breakage when host interfaces evolve.

## 4. Template Repository

A GitHub template repository (`tachyon-plugin-template`) provides the minimal scaffolding:

```
tachyon-plugin-template/
  Cargo.toml          # tachyon-plugin-sdk dependency, wasm32-unknown-unknown target
  src/
    lib.rs            # Plugin trait implementation skeleton
  tests/
    integration.rs    # Host function mocks for local testing
  plugin.toml         # Manifest with name, version, permissions
  README.md           # Build instructions, testing, publishing guide
```

Key conventions enforced in the template:

- `cargo build --target wasm32-unknown-unknown --release` produces the distributable artifact.
- `cargo test` runs unit tests natively. Integration tests use the SDK's mock host layer.
- The `plugin.toml` manifest is validated by `cargo tachyon-plugin check` (a CLI subcommand shipped with the Rust SDK).

## 5. Marketplace

### 5.1 Registry API

The plugin registry exposes a REST API:

| Method   | Endpoint                          | Description                     |
|----------|-----------------------------------|---------------------------------|
| `GET`    | `/api/v1/plugins`                 | List plugins (paginated)        |
| `GET`    | `/api/v1/plugins/:id`             | Get plugin metadata             |
| `GET`    | `/api/v1/plugins?search=:query`   | Search by name, description     |
| `POST`   | `/api/v1/plugins/:id/install`     | Download and register plugin    |
| `GET`    | `/api/v1/plugins/:id/verify`      | Check signature and CI status   |

Plugin artifacts are stored in an object store (S3-compatible). The registry database stores metadata, verification status, and download counts.

### 5.2 Verification Pipeline

CI runs on every submission to the registry:

1. **Compile**: Build `.wasm` from source with pinned toolchain versions.
2. **Test**: Execute the plugin's test suite against the current host interface.
3. **Security scan**: Static analysis of the WASM binary (detect import of non-whitelisted host functions, unreachable code patterns).
4. **Signature**: Sign the compiled `.wasm` with the registry's signing key. Clients verify the signature before loading.

### 5.3 Trust Model

- **Verified**: Passed CI and signed by the registry.
- **Community**: Submitted but not yet verified. Displayed with a warning.
- **Blocked**: Failed security scan or revoked. Refused by the runtime.

## 6. Sandboxing

The existing WASM sandbox is supplemented with explicit restrictions:

### 6.1 Filesystem

No direct host filesystem access. All data operations go through scoped host functions. The WASM module cannot import `fd_read`, `fd_write`, `path_open`, or any `wasi filesystem` API.

### 6.2 Network

No raw network access. Plugins that need external data use host functions like `fetch_url` (if the `network` permission scope is granted). The host function layer enforces URL allowlists and response size limits.

### 6.3 API Enforcement

Host functions check the plugin's declared permissions before execution. Permission checks are enforced in the runtime, not in the SDK. A malicious plugin that bypasses the SDK and calls host functions directly is still bounded by permission declarations.

### 6.4 Memory

`wasmtime` is configured with:

- **Memory limit**: 256 MB per plugin instance (configurable per deployment).
- **Fuel metering**: Instruction count limit per `execute` call. Prevents infinite loops and CPU exhaustion.
- **Instance timeout**: Wall-clock timeout (default 30 seconds) enforced by the runtime.

## 7. Implementation Priority

| Phase | Deliverable                     | Estimated Duration |
|-------|---------------------------------|--------------------|
| 1     | WIT interface stabilization     | 3 days             |
| 2     | Rust SDK (crate + docs)         | 5 days             |
| 3     | TypeScript SDK (npm + docs)     | 5 days             |
| 4     | Template repository             | 1 day              |
| 5     | Marketplace MVP (API + storage) | 5 days             |
| 6     | CI verification pipeline        | 3 days             |
| 7     | Sandboxing hardening            | 3 days             |

Total: approximately 4 weeks for a single contributor. Phases 1-3 can be parallelized.

## 8. Open Questions

- **Dependency isolation**: Should plugins be allowed to depend on shared libraries (other `.wasm` modules), or must each plugin be a fully self-contained artifact? Component Model may address this.
- **Hot-reload**: Can plugins be loaded, unloaded, and replaced without restarting the host? This requires careful state management and is deferred beyond the initial SDK release.
- **Cross-plugin communication**: Should plugins be able to call each other? This introduces trust and ordering concerns. A message-passing model with the host as intermediary is one option, but adds complexity.
- **Multi-tenant isolation**: In a hosted deployment, should each tenant get isolated plugin instances? The current model uses a single plugin registry per process.
