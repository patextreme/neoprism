{ pkgs }:

{
  default = import ./neoprism.nix { inherit pkgs; };
  cardano = import ./cardano.nix { inherit pkgs; };
}
