{
  description = "Tachyon: Rust + Tauri + Bun Dev Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    rust-analyzer-src = {
      url = "github:rust-lang/rust-analyzer";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, rust-analyzer-src }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };

        # Use stable Rust toolchain from fenix with WASM target
        rustToolchain = fenix.packages.${system}.stable.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
        ];

        # Add WASM target - use stable instead of nightly
        rustToolchainWithWasm = fenix.packages.${system}.combine [
          rustToolchain
          fenix.packages.${system}.targets.wasm32-unknown-unknown.stable.rust-std
        ];

        # Rust Analyzer from fenix
        rustAnalyzer = fenix.packages.${system}.stable.rust-analyzer;

        # System libraries required by Tauri/Git2/OpenSSL
        libraries = with pkgs; [
          webkitgtk_4_1
          gtk3
          cairo
          gdk-pixbuf
          glib
          dbus
          openssl
          librsvg
          libsoup_3
          pango
          atk
          stdenv.cc.cc.lib
          bzip2
          zlib
          libglvnd          # EGL/GL dispatch (needed for NVIDIA + Wayland)
        ];

        # Build tools
        buildTools = with pkgs; [
          curl
          wget
          pkg-config
          cmake
          perl
          python3
        ];

        # Development tools
        devTools = with pkgs; [
          git
          sqlite
          ripgrep
          fd
          jq
          bat
          eza
          postgresql_16
        ];

        # Rust development tools
        rustDevTools = with pkgs; [
          cargo-audit
          cargo-deny
          cargo-outdated
          cargo-tarpaulin
          cargo-watch
          trunk
          wasm-bindgen-cli
        ];

        # JavaScript/TypeScript tooling
        jsTools = with pkgs; [
          bun
          nodejs
        ];

        allPackages = buildTools ++ devTools ++ rustDevTools ++ jsTools;

      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = allPackages ++ [ rustToolchainWithWasm rustAnalyzer ];
          buildInputs = libraries;

          # Environment variables
          RUST_SRC_PATH = "${fenix.packages.${system}.stable.rust-src}/lib/rustlib/src/rust/library";
          RUST_BACKTRACE = "1";
          CARGO_TERM_COLOR = "always";
          
          # Library paths
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libraries;
          
          # PKG_CONFIG paths
          PKG_CONFIG_PATH = with pkgs; lib.concatStringsSep ":" [
            "${openssl.dev}/lib/pkgconfig"
            "${webkitgtk_4_1.dev}/lib/pkgconfig"
            "${libsoup_3.dev}/lib/pkgconfig"
            "${gtk3.dev}/lib/pkgconfig"
          ];

          shellHook = ''
            # Bun setup
            export BUN_INSTALL="$HOME/.bun"
            export PATH="$BUN_INSTALL/bin:$PATH"
            
            # NVIDIA + Wayland: prepend /usr/lib to find NVIDIA EGL/GBM
            # libraries (libnvidia-egl-wayland.so, libEGL_nvidia.so) that
            # the nix WebKitGTK needs but LD_LIBRARY_PATH shadows.
            # Do NOT set WEBKIT_DISABLE_COMPOSITING_MODE=1 — it kills the
            # WebKit WebProcess entirely, leaving an empty window.
            export LD_LIBRARY_PATH="/usr/lib:$LD_LIBRARY_PATH"
            
            echo "------------------------------------------------------------------"
            echo "Tachyon Dev Environment"
            echo "------------------------------------------------------------------"
            echo "Rust:    $(rustc --version 2>/dev/null || echo 'Not available')"
            echo "Cargo:   $(cargo --version 2>/dev/null || echo 'Not available')"
            echo "Trunk:   $(trunk --version 2>/dev/null || echo 'Not available')"
            echo "Bun:     $(bun --version 2>/dev/null || echo 'Not available')"
            echo "Node:    $(node --version 2>/dev/null || echo 'Not available')"
            echo "OpenSSL: $(openssl version 2>/dev/null || pkg-config --modversion openssl 2>/dev/null || echo 'Not available')"
            echo "------------------------------------------------------------------"
            echo ""
            echo "To start developing:"
            echo "  cd tachyon && cargo check                    # Verify Rust code"
            echo "  cd tachyon/crates/frontend && trunk serve   # Start Leptos dev server"
            echo "  cd tachyon && cargo run -p tachyon-server    # Start backend server"
            echo "------------------------------------------------------------------"
          '';
        };

        packages.rust-toolchain = rustToolchainWithWasm;
      }
    );
}
