{ pkgs, ... }:

{
  default = pkgs.callPackage ./neoprism-checks.nix { };
}
