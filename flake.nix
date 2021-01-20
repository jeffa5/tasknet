{
  description = "TaskNet";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    with import nixpkgs { overlays = [ rust-overlay.overlay ]; system = "x86_64-linux"; };
    {
      devShell.x86_64-linux = mkShell {
        buildInputs = [
          (rust-bin.nightly.latest.rust.override {
            extensions = [ "rust-src" ];
            targets = [ "wasm32-unknown-unknown" ];
          })
          cargo-edit
          cargo-fuzz
          cargo-make
          cargo-watch
          wasm-pack
          rust-analyzer
          pkgconfig
          openssl

          rnix-lsp
          nixpkgs-fmt
        ];
      };
    };
}
