{ pkgs }:

{
  default = pkgs.callPackage ./neoprism.nix { inherit pkgs; };
  cardano = pkgs.callPackage ./cardano.nix { inherit pkgs; };
}
