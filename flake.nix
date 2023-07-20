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

    checks.${system} =
      flake-utils.lib.filterPackages system nix;

    overlays.default = _final: _prev: self.packages.${system};

    nixosModules.${system}.tasknet-server = {
      lib,
      pkgs,
      config,
      ...
    }: let
      # the final config for this module
      cfg = config.services.tasknet-server;
      cfgFile = pkgs.writeText "tasknet-server-config" (builtins.toJSON cfg.config);
    in {
      options.services.tasknet-server = {
        enable = lib.mkEnableOption "tasknet server";
        config.address = lib.mkOption {
          type = lib.types.str;
          default = "0.0.0.0";
        };
        config.port = lib.mkOption {
          type = lib.types.port;
          default = 80;
        };
        config.serve_dir = lib.mkOption {
          type = lib.types.pathInStore;
          default = self.packages.${system}.tasknet-web;
        };
        config.documents_dir = lib.mkOption {
          type = lib.types.path;
          default = "/var/lib/tasknet-server/documents";
        };
      };

      config = lib.mkIf cfg.enable {
        systemd.services.tasknet-server = {
          wantedBy = ["multi-user.target"];
          serviceConfig.Restart = "on-failure";
          serviceConfig.ExecStart = "${self.packages.${system}.tasknet-server}/bin/tasknet-server --config-file ${cfgFile}";
        };
      };
    };

    nixosConfigurations.container = nixpkgs.lib.nixosSystem {
      system = system;
      modules = [
        self.nixosModules.${system}.tasknet-server
        ({pkgs, ...}: {
          # Only allow this to boot as a container
          boot.isContainer = true;

          # Allow nginx through the firewall
          networking.firewall.allowedTCPPorts = [80];

          services.tasknet-server.enable = true;

          system.stateVersion = "23.05";
        })
      ];
    };

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
