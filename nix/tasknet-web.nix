{
  craneLibWasm,
  trunk,
  lib,
  wasm-bindgen-cli,
}: let
  pname = "tasknet-web";
  version = "0.1.0";
  deps = craneLibWasm.buildDepsOnly {
    inherit pname version;
    src = craneLibWasm.cleanCargoSource ./..;
    cargoExtraArgs = "--target wasm32-unknown-unknown -p ${pname}";
    doCheck = false;
  };
  indexHTMLFilter = path: _type: builtins.match ".*/web/index.html$" path != null;
  assetsFilter = path: _type: builtins.match ".*/web/assets/.*$" path != null;
  stylesFilter = path: _type: builtins.match ".*/web/styles/.*$" path != null;
  indexHTMLOrCargo = path: type:
    builtins.any (f: f path type) [indexHTMLFilter assetsFilter stylesFilter craneLibWasm.filterCargoSources];
  src = lib.cleanSourceWith {
    src = ./..;
    filter = indexHTMLOrCargo;
  };
in
  craneLibWasm.buildTrunkPackage {
    inherit pname version src;
    cargoArtifacts = deps;
    trunkIndexPath = "web/index.html";
  }
