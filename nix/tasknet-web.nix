{
  lib,
  craneLibWasm,
  publicUrl ? null,
  profile ? "release",
}: let
  pname = "tasknet-web";
  version = "0.1.0";
  deps = craneLibWasm.buildDepsOnly {
    inherit pname version;
    src = craneLibWasm.cleanCargoSource ./..;
    cargoExtraArgs = "--target wasm32-unknown-unknown -p ${pname}";
    doCheck = false;
    CARGO_PROFILE = profile;
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
  trunkExtraBuildArgs =
    if publicUrl != null
    then "--public-url ${publicUrl}"
    else "";
in
  craneLibWasm.buildTrunkPackage {
    inherit pname version src trunkExtraBuildArgs;
    cargoArtifacts = deps;
    trunkIndexPath = "web/index.html";
    CARGO_PROFILE = profile;
  }
