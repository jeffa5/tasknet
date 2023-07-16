{
  craneLibWasm,
  trunk,
  lib,
  wasm-bindgen-cli,
}: let
  deps = craneLibWasm.buildDepsOnly {
    src = craneLibWasm.cleanCargoSource ./..;
    cargoExtraArgs = "--target wasm32-unknown-unknown -p tasknet-web";
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
  craneLibWasm.mkCargoDerivation {
    src = src;
    buildPhaseCargoCommand = "cd web && trunk build --release";
    cargoArtifacts = deps;
    nativeBuildInputs = [trunk wasm-bindgen-cli];
    doInstallCargoArtifacts = false;
    installPhaseCommand = ''
      mkdir -p $out
      cp -r dist $out
    '';
  }
