{ pkgs ? (import ./nix/input.nix).pkgs }:

let
  rootDir = toString ./.;
  input = import ./nix/input.nix;
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
    [ git which input.rust protobuf ] ++ (builtins.attrValues scripts);
  shellHook = "";

  # envs
  RUST_LOG = "info";
}
