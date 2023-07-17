{
  lib,
  craneLibWasm,
}: let
  pname = "tasknet-web";
  version = "0.1.0";
  cargoExtraArgs = "--target wasm32-unknown-unknown -p ${pname}";
  deps = craneLibWasm.buildDepsOnly {
    inherit pname version cargoExtraArgs;
    src = craneLibWasm.cleanCargoSource ./..;
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
  craneLibWasm.cargoClippy {
    inherit pname version src cargoExtraArgs;
    cargoArtifacts = deps;
  }
