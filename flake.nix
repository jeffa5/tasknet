{
  description = "TaskNet";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    system = "x86_64-linux";
  in
    with import nixpkgs
    {
      inherit system;
      overlays = [rust-overlay.overlays.default];
    }; {
      formatter.${system} = pkgs.alejandra;
      devShells.${system}.default = mkShell {
        buildInputs = [
          (rust-bin.nightly.latest.default.override {
            extensions = ["rust-src"];
            targets = ["wasm32-unknown-unknown"];
          })
          cargo-edit
          cargo-fuzz
          cargo-make
          cargo-watch
          wasm-pack
          pkgconfig
          openssl
        ];
      };
    };
}
