{
  pkgs,
  craneLib,
}:
pkgs.lib.makeScope pkgs.newScope (self: let
  inherit (self) callPackage;
in {
  tasknet = pkgs.callPackage ./tasknet.nix {inherit craneLib;};
})
