{
  description = "A rust implementation of PRISM node";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    cardano-node.url = "github:IntersectMBO/cardano-node?ref=10.4.1";
    cardano-wallet.url = "github:cardano-foundation/cardano-wallet?ref=v2025-03-31";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      cardano-node,
      cardano-wallet,
      ...
    }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-darwin" ] (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.unfree = true;
          overlays = [
            (import rust-overlay)
            (final: prev: {
              rustUtils = prev.callPackage ./nix/rustUtils.nix { inherit rust-overlay; };
              cardano-cli = cardano-node.packages.${system}.cardano-cli;
              cardano-node = cardano-node.packages.${system}.cardano-node;
              cardano-testnet = cardano-node.packages.${system}.cardano-testnet;
              cardano-tracer = cardano-node.packages.${system}.cardano-tracer;
              cardano-wallet = cardano-wallet.packages.${system}.cardano-wallet;
            })
          ];
        };
      in
      {
        checks = import ./nix/checks/default.nix { inherit pkgs; };
        devShells = import ./nix/devShells/default.nix { inherit pkgs; };
        packages = import ./nix/packages/default.nix { inherit pkgs; };
      }
    );
}
