{ ... }:

let
  rootDir = toString ./.;
  inherit (import ./nix/input.nix) pkgs oura rust;
  scripts = rec {
    build = pkgs.writeShellScriptBin "build" ''
      ${rust}/bin/cargo fmt
      ${rust}/bin/cargo build
    '';

    clean = pkgs.writeShellScriptBin "clean" ''
      ${rust}/bin/cargo clean
    '';

    testCoverage = pkgs.writeShellScriptBin "testCoverage" ''
      ${clean}/bin/clean

      export CARGO_INCREMENTAL=0
      export RUSTFLAGS="-C instrument-coverage"
      export LLVM_PROFILE_FILE='${rootDir}/target/profraw/cargo-test-%p-%m.profraw'

      mkdir -p ${rootDir}/target/coverage/html
      mkdir -p ${rootDir}/target/profraw

      ${rust}/bin/cargo build --all-features
      ${rust}/bin/cargo test --all-features

      ${pkgs.grcov}/bin/grcov . --binary-path ${rootDir}/target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o ${rootDir}/target/coverage/html
    '';
  };
in pkgs.mkShell {
  packages = with pkgs;
    [ git which rust protobuf oura surrealdb rust-analyzer ]
    ++ (builtins.attrValues scripts);
  shellHook = "";

  # envs
  RUST_LOG = "oura=error,info";
  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
}
