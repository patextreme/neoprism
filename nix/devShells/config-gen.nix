{
  pkgs,
  mkShell,
  writeShellApplication,
}:

let
  rootDir = "$ROOT_DIR";
  scripts = {
    build = writeShellApplication {
      name = "build";
      runtimeInputs = with pkgs; [ dhall dhall-json ];
      text = ''
        cd "${rootDir}/docker/.config"
        dhall-to-yaml <<< "(./main.dhall).basic" > "${rootDir}/docker/compose.yml"
        dhall-to-yaml <<< "(./main.dhall).dbsync" > "${rootDir}/docker/compose-dbsync.yml"
      '';
    };

    format = writeShellApplication {
      name = "format";
      runtimeInputs = with pkgs; [ dhall ];
      text = ''
        cd "${rootDir}/docker/.config"
        find . | grep '\.dhall$' | xargs -I _ bash -c "echo running dhall format on _ && dhall format _"
      '';
    };
  };
in
mkShell {
  packages =
    with pkgs;
    [
      dhall
      dhall-json
      dhall-lsp-server
    ]
    ++ (builtins.attrValues scripts);

  shellHook = ''
    export ROOT_DIR=$(${pkgs.git}/bin/git rev-parse --show-toplevel)
    ${pkgs.cowsay}/bin/cowsay "Working on project root directory: ${rootDir}"
    cd "${rootDir}/docker/.config"
  '';
}
