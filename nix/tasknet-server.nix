{
  craneLib,
  pkg-config,
  openssl,
}: let
  src = craneLib.cleanCargoSource ./..;
  pname = "tasknet-server";
  version = "0.1.0";
  cargoExtraArgs = "-p ${pname}";
  deps = craneLib.buildDepsOnly {
    inherit pname version src cargoExtraArgs;
    buildInputs = [pkg-config openssl];
  };
in
  craneLib.buildPackage {
    inherit pname version src cargoExtraArgs;
    cargoArtifacts = deps;
  }
