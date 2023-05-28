{
  description = "Exers is a rust library for compiling and running code in different languages and runtimes.";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            wasmtime
            cargo
            cargo-watch
            pkg-config
            (rust-bin.stable.latest.default.override {
              targets = [ "x86_64-unknown-linux-gnu" "wasm32-unknown-unknown" "wasm32-wasi" ];
              extensions = [ "rust-analyzer" "rust-src" "rust-std" ];
            })
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      }
    );
}