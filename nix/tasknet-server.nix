{
  craneLib,
  pkg-config,
  openssl,
}: let
  src = craneLib.cleanCargoSource ./..;
  cargoExtraArgs = "-p server";
  deps = craneLib.buildDepsOnly {
    inherit src cargoExtraArgs;
    buildInputs = [pkg-config openssl];
  };
in
  craneLib.buildPackage {
    inherit src cargoExtraArgs;
    cargoArtifacts = deps;
  }
