{
  description = "A rust implementation of PRISM node";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.unfree = true;
          overlays = [ (import rust-overlay) ];
        };
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ ];
        };
      in {
        devShells.default = let
          rootDir = "$ROOT_DIR";
          scripts = {
            format = pkgs.writeShellScriptBin "format" ''
              find ${rootDir} | grep '\.nix$' | xargs -I _ bash -c "echo running nixfmt on _ && ${pkgs.nixfmt}/bin/nixfmt _"
              find ${rootDir} | grep '\.toml$' | xargs -I _ bash -c "echo running taplo on _ && ${pkgs.taplo}/bin/taplo format _"
              ${rust}/bin/cargo fmt
            '';

            build = pkgs.writeShellScriptBin "build" ''
              ${rust}/bin/cargo fmt
              ${rust}/bin/cargo build --all-features
            '';

            clean = pkgs.writeShellScriptBin "clean" ''
              ${rust}/bin/cargo clean
            '';
          };
        in pkgs.mkShell {
          packages = with pkgs;
            [
              # base
              git
              which
              openssl
              # lsp
              nil
              taplo
              # rust
              rust
              protobuf
            ] ++ (builtins.attrValues scripts);

          shellHook = ''
            export ROOT_DIR=$(${pkgs.git}/bin/git rev-parse --show-toplevel)
            ${pkgs.cowsay}/bin/cowsay "Working on project root directory: ${rootDir}"
            cd ${rootDir}
          '';

          # envs
          RUST_LOG =
            "oura=warn,sqlx::query=warn,prism_core=debug,prism_node=debug,tracing::span=warn,info";
        };
      });
}
