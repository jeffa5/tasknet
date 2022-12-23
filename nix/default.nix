{
  pkgs,
  crane,
  system,
}:
pkgs.lib.makeScope pkgs.newScope (self: let
  inherit (self) callPackage;
  craneLib = crane.lib.${system};
  rustWasm = pkgs.rust-bin.stable.latest.default.override {
    targets = ["wasm32-unknown-unknown"];
  };
  craneLibWasm = (crane.mkLib pkgs).overrideToolchain rustWasm;
in {
  tasknet-web = pkgs.callPackage ./tasknet-web.nix {inherit craneLibWasm;};
  tasknet = pkgs.callPackage ./tasknet.nix {inherit craneLib;};
})
