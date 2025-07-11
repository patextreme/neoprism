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
      text = ''
        cd "${rootDir}/docker/.config"
        ${pkgs.cue}/bin/cue export main.cue -e basic -f -o ../compose.yml
        ${pkgs.cue}/bin/cue export main.cue -e dbsync -f -o ../compose-dbsync.yml
      '';
    };

    format = writeShellApplication {
      name = "format";
      text = ''
        cd "${rootDir}/docker/.config"
        find . | grep '\.cue$' | xargs -I _ bash -c "echo running cue fmt on _ && ${pkgs.cue}/bin/cue fmt _"
      '';
    };
  };
in
mkShell {
  packages =
    with pkgs;
    [
      # cue
      cue
      cuelsp
    ]
    ++ (builtins.attrValues scripts);

  shellHook = ''
    export ROOT_DIR=$(${pkgs.git}/bin/git rev-parse --show-toplevel)
    ${pkgs.cowsay}/bin/cowsay "Working on project root directory: ${rootDir}"
    cd "${rootDir}/docker/.config"
  '';
}
