{
  pkgs,
  rustUtils,
  mkShell,
  writeShellApplication,
}:

let
  rootDir = "$ROOT_DIR";
  rust = rustUtils.rust;
  localDb = {
    port = 5432;
    username = "postgres";
    password = "postgres";
    dbName = "postgres";
  };
  scripts = rec {
    format = writeShellApplication {
      name = "format";
      runtimeInputs = with pkgs; [
        nixfmt-rfc-style
        taplo
      ];
      text = ''
        cd "${rootDir}"
        find . | grep '\.nix$' | xargs -I _ bash -c "echo running nixfmt on _ && nixfmt _"
        find . | grep '\.toml$' | xargs -I _ bash -c "echo running taplo on _ && taplo format _"

        cd "${rootDir}/lib/indexer-storage/migrations"
        sqlfluff fix .
        sqlfluff lint .

        cd "${rootDir}/docker/.config"
        find . | grep '\.dhall$' | xargs -I _ bash -c "echo running dhall format on _ && dhall format _"

        cd "${rootDir}"
        cargo fmt
      '';
    };

    buildAssets = writeShellApplication {
      name = "buildAssets";
      text = ''
        cd "${rootDir}/service/indexer-node"
        tailwindcss -i tailwind.css -o ./assets/styles.css
      '';
    };

    buildConfig = writeShellApplication {
      name = "buildConfig";
      text = ''
        cd "${rootDir}/docker/.config"
        dhall-to-yaml <<< "(./main.dhall).basic" > "${rootDir}/docker/compose.yml"
        dhall-to-yaml <<< "(./main.dhall).dbsync" > "${rootDir}/docker/compose-dbsync.yml"
      '';
    };

    build = writeShellApplication {
      name = "build";
      text = ''
        cd "${rootDir}"
        ${buildAssets}/bin/buildAssets
        ${buildConfig}/bin/buildConfig
        cargo build --all-features
      '';
    };

    clean = writeShellApplication {
      name = "clean";
      text = ''
        cd "${rootDir}"
        cargo clean
      '';
    };

    dbUp = writeShellApplication {
      name = "dbUp";
      text = ''
        docker run \
          -d --rm \
          --name prism-db \
          -e POSTGRES_DB=${localDb.dbName} \
          -e POSTGRES_USER=${localDb.username} \
          -e POSTGRES_PASSWORD=${localDb.password} \
          -p ${toString localDb.port}:5432 postgres:16
      '';
    };

    dbDown = writeShellApplication {
      name = "dbDown";
      text = ''
        docker stop prism-db
      '';
    };

    pgDump = writeShellApplication {
      name = "pgDump";
      runtimeInputs = with pkgs; [ postgresql_16 ];
      text = ''
        cd "${rootDir}"
        export PGPASSWORD=${localDb.password}
        pg_dump -h localhost -p ${toString localDb.port} -U ${localDb.username} -w -d ${localDb.dbName} -Fc > postgres.dump
      '';
    };

    pgRestore = writeShellApplication {
      name = "pgRestore";
      runtimeInputs = with pkgs; [ postgresql_16 ];
      text = ''
        cd "${rootDir}"
        export PGPASSWORD=${localDb.password}
        pg_restore -h localhost -p ${toString localDb.port} -U ${localDb.username} -w -d ${localDb.dbName} postgres.dump
      '';
    };

    runNode = writeShellApplication {
      name = "runNode";
      text = ''
        cd "${rootDir}"
        ${buildAssets}/bin/buildAssets
        cargo run --bin indexer-node -- --db-url postgres://${localDb.username}:${localDb.password}@localhost:${toString localDb.port}/${localDb.dbName} "$@"
      '';
    };

    bumpVersion = writeShellApplication {
      name = "bumpVersion";
      text = ''
        cd "${rootDir}"
        NEW_VERSION=$(${pkgs.git-cliff}/bin/git-cliff --bump --context | ${pkgs.jq}/bin/jq -r .[0].version | sed s/^v//)
        ${setVersion}/bin/setVersion "$NEW_VERSION"
      '';
    };

    setVersion = writeShellApplication {
      name = "setVersion";
      text = ''
        cd "${rootDir}"
        NEW_VERSION=$1
        echo "Setting new version to $NEW_VERSION"
        echo "$NEW_VERSION" > version
        ${rust}/bin/cargo set-version "$NEW_VERSION"
        ${buildConfig}/bin/buildConfig
        ${pkgs.git-cliff}/bin/git-cliff -t "$NEW_VERSION" > CHANGELOG.md
      '';
    };
  };
in
mkShell {
  packages =
    with pkgs;
    [
      # base
      docker
      git
      git-cliff
      less
      ncurses
      protobuf
      watchexec
      which
      # config
      dhall
      dhall-json
      # db
      sqlfluff
      sqlx-cli
      # rust
      cargo-edit
      cargo-expand
      cargo-license
      cargo-udeps
      rust
      # node
      nodejs_20
      tailwindcss_4
    ]
    ++ (builtins.attrValues scripts);

  shellHook = ''
    export ROOT_DIR=$(${pkgs.git}/bin/git rev-parse --show-toplevel)
    ${pkgs.cowsay}/bin/cowsay "Working on project root directory: ${rootDir}"
    cd "${rootDir}"
  '';

  # envs
  RUST_LOG = "info,oura=warn,tower_http::trace=debug";
}
