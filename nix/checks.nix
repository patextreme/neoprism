{ pkgs, rust, ... }:

let
  rustPlatform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
in
{
  default = rustPlatform.buildRustPackage {
    name = "neoprism-checks";
    src = pkgs.lib.cleanSource ./..;
    cargoLock.lockFile = ../Cargo.lock;
    nativeBuildInputs = with pkgs; [
      protobuf
      sqlfluff
    ];
    buildPhase = "cargo b --all-features --all-targets";
    checkPhase = ''
      sqlfluff lint --dialect postgres ./prism-storage/migrations
      cargo fmt --check
      cargo clippy --all-features --all-targets -- -D warnings
      cargo test --all-features
    '';
    installPhase = "touch $out";

    PROTOC = "${pkgs.protobuf}/bin/protoc";
  };
}
