{ pkgs ? (import ./nix/input.nix).pkgs }:

let
  rootDir = toString ./.;
  input = import ./nix/input.nix;
  oura = pkgs.rustPlatform.buildRustPackage rec {
    name = "oura";
    src = pkgs.fetchgit {
      url = "https://github.com/txpipe/oura.git";
      rev = "v1.8.1";
      sha256 = "HSVVrhwPPUeAHiIx/16r86pKQtCsNIQSVcTH92cLdNE=";
    };
    cargoHash = "sha256-a+j40vo/xzHx64pRdmN8gIDhXCi5xgrbWp9Bx15EXbU=";
    buildNoDefaultFeatures = true;
    buildFeatures = [ "logs" ];
  };
  scripts = rec {
    build = pkgs.writeShellScriptBin "build" ''
      ${input.rust}/bin/cargo fmt
      ${input.rust}/bin/cargo build
    '';

    clean = pkgs.writeShellScriptBin "clean" ''
      ${input.rust}/bin/cargo clean
    '';
  };
in pkgs.mkShell {
  packages = with pkgs;
    [ git which input.rust protobuf oura ] ++ (builtins.attrValues scripts);
  shellHook = "";

  # envs
  RUST_LOG = "info";
}
