{
  description = "Tachyon Dev Environment";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = { url = "github:nix-community/fenix"; inputs.nixpkgs.follows = "nixpkgs"; };
  };
  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rustToolchain = fenix.packages.${system}.stable.withComponents [ "cargo" "clippy" "rust-src" "rustc" "rustfmt" "llvm-tools-preview" ];
        wasmTarget = fenix.packages.${system}.targets.wasm32-unknown-unknown.latest.rust-std;
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ pkg-config openssl cmake rustToolchain wasmTarget nodePackages.tailwindcss cargo-leptos cargo-tauri ];
          buildInputs = with pkgs; [ openssl sqlite dbus glib gtk3 libsoup webkitgtk librsvg ];
          shellHook = ''
            export WEBKIT_DISABLE_COMPOSITING_MODE=1
            echo "Tachyon Environment Loaded"
          '';
        };
      }
    );
}
