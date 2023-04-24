{ pkgs ? (import ./nix/input.nix).pkgs }:

let
  rootDir = toString ./.;
  input = import ./nix/input.nix;
  scripts = rec { };
in pkgs.mkShell {
  packages = with pkgs;
    [ git which input.rust ] ++ (builtins.attrValues scripts);
  shellHook = "";

  # envs
  RUST_LOG = "info";
}
