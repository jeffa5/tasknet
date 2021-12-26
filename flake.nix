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
        lib = pkgs.lib;
        rust = pkgs.rust-bin.nightly.latest.rust;
        cargoNix = import ./Cargo.nix {
          inherit pkgs;
          release = true;
        };
        docker-repo = "jeffas";
      in
      {
        packages = lib.attrsets.mapAttrs
          (name: value: value.build)
          cargoNix.workspaceMembers // {
          docker-server = pkgs.dockerTools.buildImage {
            name = "${docker-repo}/tasknet-server";
            tag = "${cargoNix.internal.crates.tasknet-server.version}";
            config = {
              Entrypoint = [ "${self.packages.${system}.tasknet-server}/bin/tasknet-server" ];
            };
          };
        };

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

            flyway
            postgresql

            crate2nix
            docker-compose

            cfssl

            kind
            kubectl
            k9s
            kubernetes-helm
            skaffold

            nodejs

            rnix-lsp
            nixpkgs-fmt
          ];
          DOCKER_BUILDKIT = 1;
        };
      });
}
