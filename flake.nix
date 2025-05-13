{
  description = "A rust implementation of PRISM node";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.unfree = true;
          overlays = [ (import rust-overlay) ];
        };
        nightlyVersion = "2025-04-23";
        rustMinimal = pkgs.rust-bin.nightly.${nightlyVersion}.minimal;
        rust = pkgs.rust-bin.nightly.${nightlyVersion}.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
          targets = [ ];
        };
      in
      {
        devShells = import ./nix/devShells.nix { inherit pkgs rust; };
        checks = import ./nix/checks.nix { inherit pkgs rust; };
        packages = import ./nix/packages.nix {
          inherit pkgs;
          rust = rustMinimal;
        };
      }
    );
}
