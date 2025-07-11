{ pkgs }:

{
  default = pkgs.callPackage ./neoprism.nix { inherit pkgs; };
  config-gen = pkgs.callPackage ./config-gen.nix { inherit pkgs; };
}
