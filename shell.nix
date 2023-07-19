{ ... }:

let
  rootDir = toString ./.;
  inherit (import ./nix/input.nix) pkgs oura rust;
  scripts = rec {
    format = pkgs.writeShellScriptBin "format" ''
      ${pkgs.nixfmt}/bin/nixfmt *.nix
      ${pkgs.nixfmt}/bin/nixfmt ${rootDir}/nix/*.nix
      ${rust}/bin/cargo fmt
    '';

    build = pkgs.writeShellScriptBin "build" ''
      ${rust}/bin/cargo fmt
      ${rust}/bin/cargo build --all-features
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

    generateEntity = pkgs.writeShellScriptBin "generateEntity" ''
      mkdir -p ${rootDir}/target
      rm -f ${rootDir}/target/tmp.db
      touch ${rootDir}/target/tmp.db
      ${rust}/bin/cargo run -p prism-persistence --no-default-features -- "sqlite://${rootDir}/target/tmp.db"
      ${pkgs.sea-orm-cli}/bin/sea-orm-cli generate entity \
        --database-url sqlite://${rootDir}/target/tmp.db \
        -o ${rootDir}/prism-persistence/src/entity
    '';
  };
in pkgs.mkShell {
  packages = with pkgs;
    [ git which rust protobuf oura sea-orm-cli ]
    ++ (builtins.attrValues scripts);
  shellHook = "";

  # envs
  RUST_LOG =
    "oura=warn,sqlx::query=warn,prism_core=debug,prism_node=debug,info";
}
