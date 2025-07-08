{
  lib,
  makeRustPlatform,
  rust,
  cargoLock,
  buildPackages,
}:

let
  rustPlatform = makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
in
rustPlatform.buildRustPackage {
  inherit cargoLock;
  name = "neoprism";
  src = lib.cleanSource ./../..;
  nativeBuildInputs = with buildPackages; [ protobuf ];
  doCheck = false;
  PROTOC = "${buildPackages.protobuf}/bin/protoc";
}
