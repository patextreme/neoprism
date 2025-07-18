{
  description = "A rust implementation of PRISM node";

  nixConfig = {
    extra-substituters = [ "https://cache.iog.io" ];
    extra-trusted-public-keys = [ "hydra.iohk.io:f/Ea+s+dFdN+3Y/G+FDgSq+a5NEWhJGzdjvKNGv0/EQ=" ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    cardano-node.url = "github:IntersectMBO/cardano-node?ref=10.4.1";
    cardano-db-sync.url = "github:IntersectMBO/cardano-db-sync?ref=13.6.0.5";
    cardano-wallet.url = "github:cardano-foundation/cardano-wallet?ref=v2025-03-31";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      cardano-node,
      cardano-db-sync,
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
              cardano-wallet = cardano-wallet.packages.${system}.cardano-wallet;
              cardano-db-sync = cardano-db-sync.packages.${system}.default;
            })
          ];
        };
      in
      {
        apps = {
          publish-testnet-image = {
            type = "app";
            program =
              (pkgs.writeShellApplication {
                name = "publish";
                runtimeInputs = with pkgs; [
                  nix
                  docker
                ];
                text = ''
                  TAG=$(date +"%Y%m%d-%H%M%S")
                  nix build .#cardano-testnet-docker-linux-amd64 -o result-amd64
                  nix build .#cardano-testnet-docker-linux-arm64 -o result-arm64
                  docker load < ./result-amd64
                  docker load < ./result-arm64
                  docker tag cardano-testnet:latest-amd64 "patextreme/cardano-testnet:$TAG-amd64"
                  docker tag cardano-testnet:latest-arm64 "patextreme/cardano-testnet:$TAG-arm64"

                  rm -rf ./result-amd64
                  rm -rf ./result-arm64

                  docker push "patextreme/cardano-testnet:$TAG-amd64"
                  docker push "patextreme/cardano-testnet:$TAG-arm64"

                  # create multi-arch image
                  docker manifest create  "patextreme/cardano-testnet:$TAG" \
                    "patextreme/cardano-testnet:$TAG-amd64" \
                    "patextreme/cardano-testnet:$TAG-arm64"
                  docker manifest push "patextreme/cardano-testnet:$TAG"
                '';
              }).outPath
              + "/bin/publish";
          };
        };

        checks = import ./nix/checks/default.nix { inherit pkgs; };
        devShells = import ./nix/devShells/default.nix { inherit pkgs; };
        packages = import ./nix/packages/default.nix { inherit pkgs; };
      }
    );
}
