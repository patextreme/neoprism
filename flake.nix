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
          npmDeps = pkgs.buildNpmPackage {
            name = "assets-nodemodules";
            src = ./.;
            npmDepsHash = "sha256-snC2EOnV3200x4fziwcj/1o9KoqSJkTFgJgAh9TWNpE=";
            dontNpmBuild = true;
            installPhase = ''
              runHook preInstall
              cp -r ./node_modules $out
              runHook postInstall
            '';
          };

          assets = pkgs.stdenv.mkDerivation {
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

          default = rustPlatformMinimal.buildRustPackage {
            name = "neoprism";
            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.protobuf ];
            doCheck = false;
            PROTOC = "${pkgs.protobuf}/bin/protoc";
          };

          dockerImage = pkgs.dockerTools.buildLayeredImage {
            name = "neoprism";
            tag = "0.1.0-SNAPSHOT";
            created = "now";
            contents = [ default assets ];
            config = {
              Env = [ "RUST_LOG=info,oura=warn,tracing::span=warn" ];
              Entrypoint = [ "/bin/prism-node" ];
              Cmd = [
                "--assets"
                "/assets"
              ];
              WorkingDir = "";
            };
          };
        };

        devShells.default =
          let
            rootDir = "$ROOT_DIR";
            localDb = {
              port = 5432;
              username = "postgres";
              password = "postgres";
              dbName = "postgres";
            };
            scripts = rec {
              format = pkgs.writeShellScriptBin "format" ''
                cd ${rootDir}
                find . | grep '\.nix$' | xargs -I _ bash -c "echo running nixfmt on _ && ${pkgs.nixfmt-rfc-style}/bin/nixfmt _"
                find . | grep '\.toml$' | xargs -I _ bash -c "echo running taplo on _ && ${pkgs.taplo}/bin/taplo format _"

                ${pkgs.sqlfluff}/bin/sqlfluff fix ./prism-storage/migrations
                ${pkgs.sqlfluff}/bin/sqlfluff lint ./prism-storage/migrations

                ${pkgs.dioxus-cli}/bin/dx fmt
                ${rust}/bin/cargo fmt
              '';

              buildAssets = pkgs.writeShellScriptBin "buildAssets" ''
                cd ${rootDir}/prism-node
                ${pkgs.tailwindcss_4}/bin/tailwindcss -i tailwind.css -o ./assets/styles.css
              '';

              build = pkgs.writeShellScriptBin "build" ''
                cd ${rootDir}
                ${buildAssets}/bin/buildAssets
                ${rust}/bin/cargo build --all-features
              '';

              clean = pkgs.writeShellScriptBin "clean" ''
                ${rust}/bin/cargo clean
              '';

              dbUp = pkgs.writeShellScriptBin "dbUp" ''
                ${pkgs.docker}/bin/docker run \
                  -d --rm \
                  --name prism-db \
                  -e POSTGRES_DB=${localDb.dbName} \
                  -e POSTGRES_USER=${localDb.username} \
                  -e POSTGRES_PASSWORD=${localDb.password} \
                  -p ${toString localDb.port}:5432 postgres:16
              '';

              dbDown = pkgs.writeShellScriptBin "dbDown" ''
                ${pkgs.docker}/bin/docker stop prism-db
              '';

              pgDump = pkgs.writeShellScriptBin "pgDump" ''
                export PGPASSWORD=${localDb.password}
                ${pkgs.postgresql_16}/bin/pg_dump -h localhost -p ${toString localDb.port} -U ${localDb.username} -w -d ${localDb.dbName} -Fc > ${rootDir}/postgres.dump
              '';

              pgRestore = pkgs.writeShellScriptBin "pgRestore" ''
                export PGPASSWORD=${localDb.password}
                ${pkgs.postgresql_16}/bin/pg_restore -h localhost -p ${toString localDb.port} -U ${localDb.username} -w -d ${localDb.dbName} ${rootDir}/postgres.dump
              '';

              migrate = pkgs.writeShellScriptBin "migrate" ''
                ${pkgs.sea-orm-cli}/bin/sea-orm-cli migrate up -d prism-migration --database-url postgres://${localDb.username}:${localDb.password}@localhost:${toString localDb.port}/${localDb.dbName}
              '';

              runNode = pkgs.writeShellScriptBin "runNode" ''
                cd ${rootDir}
                ${buildAssets}/bin/buildAssets
                ${rust}/bin/cargo run --bin prism-node -- --db postgres://${localDb.username}:${localDb.password}@localhost:${toString localDb.port}/${localDb.dbName} "$@"
              '';
            };
          in
          pkgs.mkShell {
            packages =
              with pkgs;
              [
                # base
                docker
                git
                less
                ncurses
                protobuf
                watchexec
                which
                # db
                sqlfluff
                sqlx-cli
                # rust
                cargo-edit
                cargo-expand
                cargo-license
                cargo-udeps
                protobuf
                rust
                # node
                nodejs_20
                tailwindcss_4
              ]
              ++ (builtins.attrValues scripts);

            shellHook = ''
              export ROOT_DIR=$(${pkgs.git}/bin/git rev-parse --show-toplevel)
              ${pkgs.cowsay}/bin/cowsay "Working on project root directory: ${rootDir}"
              cd ${rootDir}
            '';

            # envs
            RUST_LOG = "info,oura=warn,tracing::span=warn";
          };

        devShells.diagrams = pkgs.mkShell {
          packages = with pkgs; [
            graphviz
            uv
            ruff
            python312
            (python312.withPackages (p: with p; [ python-lsp-server ]))
          ];

          shellHook = ''
            export ROOT_DIR=$(${pkgs.git}/bin/git rev-parse --show-toplevel)
            cd $ROOT_DIR/docs/diagrams
          '';
        };
      }
    );
}
