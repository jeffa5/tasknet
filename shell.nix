let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
in
with nixpkgs;
stdenv.mkDerivation {
  name = "moz_overlay_shell";
  buildInputs = [
    (nixpkgs.latest.rustChannels.nightly.rust.override {
      targets = [ "wasm32-unknown-unknown" ];
      extensions = [ "rust-src" ];
    })
    cargo-make
    cargo-watch
    wasm-pack
    rust-analyzer
    pkgconfig
    openssl

    rnix-lsp
    nixpkgs-fmt
  ];
}
