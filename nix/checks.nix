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
    cargoLock = (import ./cargo.nix).cargoLock;
    nativeBuildInputs = with pkgs; [
      protobuf
      sqlfluff
    ];
    buildPhase = "cargo b --all-features --all-targets";
    checkPhase = ''
      sqlfluff lint --dialect postgres ./indexer-storage/migrations
      cargo fmt --check
      cargo clippy --all-features --all-targets -- -D warnings
      cargo test --all-features

      # check individual feature
      echo "checking feature gate for identus-apollo"
      cargo clippy -p identus-apollo --all-targets -- -D warnings
      cargo clippy -p identus-apollo --all-targets --features base64 -- -D warnings
      cargo clippy -p identus-apollo --all-targets --features ed25519 -- -D warnings
      cargo clippy -p identus-apollo --all-targets --features hash -- -D warnings
      cargo clippy -p identus-apollo --all-targets --features hex -- -D warnings
      cargo clippy -p identus-apollo --all-targets --features jwk -- -D warnings
      cargo clippy -p identus-apollo --all-targets --features openapi -- -D warnings
      cargo clippy -p identus-apollo --all-targets --features secp256k1 -- -D warnings
      cargo clippy -p identus-apollo --all-targets --features serde -- -D warnings
      cargo clippy -p identus-apollo --all-targets --features x25519 -- -D warnings

      echo "checking feature gate for identus-did-core"
      cargo clippy -p identus-did-core --all-targets -- -D warnings
      cargo clippy -p identus-did-core --all-targets --features openapi -- -D warnings

      echo "checking feature gate for identus-did-prism"
      cargo clippy -p identus-did-prism --all-targets -- -D warnings
      cargo clippy -p identus-did-prism --all-targets --features oura -- -D warnings
    '';
    installPhase = "touch $out";

    PROTOC = "${pkgs.protobuf}/bin/protoc";
  };
}
