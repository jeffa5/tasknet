{
  craneLib,
  pkg-config,
  openssl,
}:
craneLib.buildPackage {
  src = craneLib.cleanCargoSource ./..;
  buildInputs = [pkg-config openssl];
}
