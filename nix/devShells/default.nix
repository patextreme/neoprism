{ pkgs }:

{
  default = pkgs.callPackage ./neoprism.nix { inherit pkgs; };
  testnet-local = pkgs.callPackage ./testnet-local.nix { inherit pkgs; };
}
