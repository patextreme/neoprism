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
        rust = pkgs.rust-bin.nightly."2024-07-04".default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
          targets = [ ];
        };
      in
      {
        devShells.default =
          let
            rootDir = "$ROOT_DIR";
            localDb = {
              port = 5432;
              username = "postgres";
              password = "postgres";
              dbName = "postgres";
            };
            scripts = {
              format = pkgs.writeShellScriptBin "format" ''
                cd ${rootDir}
                find ${rootDir} | grep '\.nix$' | xargs -I _ bash -c "echo running nixfmt on _ && ${pkgs.nixfmt-rfc-style}/bin/nixfmt _"
                find ${rootDir} | grep '\.toml$' | xargs -I _ bash -c "echo running taplo on _ && ${pkgs.taplo}/bin/taplo format _"
                ${pkgs.dioxus-cli}/bin/dx fmt
                ${rust}/bin/cargo fmt
              '';

              build = pkgs.writeShellScriptBin "build" ''
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

              migrate = pkgs.writeShellScriptBin "migrate" ''
                ${pkgs.sea-orm-cli}/bin/sea-orm-cli migrate up -d prism-migration --database-url postgres://postgres:postgres@localhost:5432/postgres
              '';

              generateEntity = pkgs.writeShellScriptBin "generateEntity" ''
                rm -rf prism-storage/src/entity
                ${pkgs.sea-orm-cli}/bin/sea-orm-cli generate entity \
                  --database-url postgres://postgres:postgres@localhost:5432/postgres \
                  -o prism-storage/src/entity \
                  --date-time-crate time
              '';

              runServer = pkgs.writeShellScriptBin "runServer" ''
                ${rust}/bin/cargo run --bin prism-node -- server --cardano_addr localhost:3000 --db postgres://${localDb.username}:${localDb.password}@localhost:${toString localDb.port}/${localDb.dbName}
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
                which
                # lsp
                nil
                taplo
                # rust
                cargo-edit
                cargo-license
                cargo-udeps
                cargo-watch
                dioxus-cli
                protobuf
                rust
                sea-orm-cli
                # tailwind & html
                nodejs_20
                nodePackages."@tailwindcss/language-server"
                nodePackages.vscode-langservers-extracted
              ]
              ++ (builtins.attrValues scripts);

            shellHook = ''
              export ROOT_DIR=$(${pkgs.git}/bin/git rev-parse --show-toplevel)
              ${pkgs.cowsay}/bin/cowsay "Working on project root directory: ${rootDir}"
              cd ${rootDir}
            '';

            # envs
            RUST_LOG = "oura=warn,sqlx::query=warn,prism_core=debug,prism_node=debug,tracing::span=warn,info";
          };
      }
    );
}
