{
  description = "TaskNet";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          overlays = [ rust-overlay.overlay ];
          system = "x86_64-linux";
        };
        rust = pkgs.rust-bin.nightly.latest.rust;
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            (rust.override {
              extensions = [ "rust-src" ];
              targets = [ "wasm32-unknown-unknown" ];
            })
            cargo-edit
            cargo-fuzz
            cargo-make
            cargo-watch
            wasm-pack
            pkgconfig
            openssl

            rnix-lsp
            nixpkgs-fmt
          ];
        };
      });
}
