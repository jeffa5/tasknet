{
  pkgs,
  crane,
  system,
}:
pkgs.lib.makeScope pkgs.newScope (self: let
  craneLib = crane.lib.${system};
  rustWasm = pkgs.rust-bin.stable.latest.default.override {
    targets = ["wasm32-unknown-unknown"];
  };
  craneLibWasm = (crane.mkLib pkgs).overrideToolchain rustWasm;
in rec {
  tasknet-web = self.callPackage ./tasknet-web.nix {inherit craneLibWasm;};
  tasknet-web-github = tasknet-web.override {publicUrl = "/tasknet";};
  tasknet-server = self.callPackage ./tasknet-server.nix {inherit craneLib;};

  # checks
  format = craneLib.cargoFmt {
    pname = "tasknet";
    version = "0.1.0";
    src = ../.;
  };
  server-clippy = self.callPackage ./tasknet-server-clippy.nix {inherit craneLib;};
  web-clippy = self.callPackage ./tasknet-web-clippy.nix {inherit craneLibWasm;};
  server-test = self.callPackage ./tasknet-server-test.nix {inherit craneLib;};
})
