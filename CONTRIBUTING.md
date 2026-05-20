# Contributing to Tachyon

See [tachyon/CONTRIBUTING.md](tachyon/CONTRIBUTING.md) for the full contribution guide.

## Quick Start

```bash
git clone https://github.com/WyattAu/Tachyon.git
cd Tachyon/tachyon
cp .env.example .env
# Edit .env with your DATABASE_URL and JWT_SECRET
cargo run -p tachyon-server -- migrate
cargo run -p tachyon-server
```

## Pre-Commit Hooks

Install quality gates:

```bash
git config core.hooksPath .githooks
```

This enforces: cargo fmt, cargo clippy, unit tests, rustdoc, secret detection.
