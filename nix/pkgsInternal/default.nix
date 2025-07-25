{ pkgs }:

{
  prism-cli = pkgs.callPackage ./prism-cli.nix { };
  prism-proto-jar = pkgs.callPackage ./prism-proto-jar { };
}
