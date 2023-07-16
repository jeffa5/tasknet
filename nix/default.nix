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
in {
  tasknet-web = self.callPackage ./tasknet-web.nix {inherit craneLibWasm;};
  tasknet-server = self.callPackage ./tasknet-server.nix {inherit craneLib;};
  tasknet-server-docker = self.callPackage ./tasknet-server-docker.nix {};
})
