{
  description = "Flake for gym-rs";

  inputs = {
    nixpks.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust = (
          pkgs.rust-bin.stable."1.82.0".default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
            targets = ["x86_64-unknown-linux-gnu"];
          }
        );
      in
        with pkgs; {
          devShells.default = mkShell {
            buildInputs = [
              # System dependencies
              cmake     # Required by `SDL2`
              SDL2_gfx  # Used in tests

              # Rust toolchain
              # Nightly rustfmt because some rules are not supported in stable.
              (lib.hiPrio rust-bin.nightly."2024-09-01".rustfmt)
              rust

              # Tooling
              taplo     # Formats `Cargo.toml`
              alejandra # Formats nix files.
            ];
          };
        }
    );
}
