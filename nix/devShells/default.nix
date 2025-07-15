{ pkgs }:

{
  default = pkgs.callPackage ./neoprism.nix { inherit pkgs; };
}
