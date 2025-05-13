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
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rust;
          rustc = rust;
        };
        rustPlatformMinimal = pkgs.makeRustPlatform {
          cargo = rustMinimal;
          rustc = rustMinimal;
        };
      in
      {
        devshells = pkgs.callPackage ./nix/devShells { };

        checks = {
          default = rustPlatform.buildRustPackage {
            name = "neoprism-checks";
            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = with pkgs; [
              protobuf
              sqlfluff
            ];
            buildPhase = "cargo b --all-features --all-targets";
            checkPhase = ''
              sqlfluff lint --dialect postgres ./prism-storage/migrations
              cargo fmt --check
              cargo clippy --all-features --all-targets -- -D warnings
              cargo test --all-features
            '';
            installPhase = "touch $out";

            PROTOC = "${pkgs.protobuf}/bin/protoc";
          };
        };

        packages = rec {
          resolver-ui-assets =
            let
              npmDeps = pkgs.buildNpmPackage {
                name = "assets-nodemodules";
                src = ./.;
                npmDepsHash = "sha256-snC2EOnV3200x4fziwcj/1o9KoqSJkTFgJgAh9TWNpE=";
                dontNpmBuild = true;
                installPhase = ''
                  cp -r ./node_modules $out
                '';
              };
            in
            pkgs.stdenv.mkDerivation {
              name = "assets";
              src = ./.;
              buildInputs = with pkgs; [ tailwindcss_4 ];
              installPhase = ''
                mkdir -p ./node_modules
                cp -r ${npmDeps}/* ./node_modules
                cd prism-node
                mkdir -p $out/assets
                tailwindcss -i ./tailwind.css -o $out/assets/styles.css
              '';
            };

          resolver-bin = rustPlatformMinimal.buildRustPackage {
            name = "neoprism";
            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.protobuf ];
            doCheck = false;
            PROTOC = "${pkgs.protobuf}/bin/protoc";
          };

          resolver-docker = pkgs.dockerTools.buildLayeredImage {
            name = "neoprism";
            tag = "0.1.0-SNAPSHOT";
            created = "now";
            contents = [
              resolver-bin
              resolver-ui-assets
            ];
            config = {
              Env = [ "RUST_LOG=info,oura=warn" ];
              Entrypoint = [ "/bin/prism-node" ];
              Cmd = [
                "--assets"
                "/assets"
              ];
              WorkingDir = "";
            };
          };
        };
      }
    );
}
