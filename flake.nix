{
  description = "TaskNet";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
    crane.url = "github:ipetkov/crane";
    crane.inputs.rust-overlay.follows = "rust-overlay";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    crane.inputs.flake-utils.follows = "flake-utils";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    crane,
    flake-utils,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [rust-overlay.overlays.default];
    };
    nix = import ./nix {inherit pkgs crane system;};
  in {
    packages.${system} =
      flake-utils.lib.filterPackages system nix;

    overlays.default = _final: _prev: self.packages.${system};

    apps.${system} = {
      tasknet-server = {
        type = "app";
        program = "${nix.tasknet-server}/bin/tasknet-server";
      };
      default = self.apps.${system}.tasknet-server;
    };

    formatter.${system} = pkgs.alejandra;

    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        (rust-bin.stable.latest.default.override {
          extensions = ["rust-src"];
          targets = ["wasm32-unknown-unknown"];
        })
        trunk
      ];
    };
  };
}
