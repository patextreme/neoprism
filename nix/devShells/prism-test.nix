{ pkgs }:

let
  rootDir = "$ROOT_DIR";
in
pkgs.mkShell {
  packages = with pkgs; [
    git
    jdk
    metals
    scala-cli
  ];

  shellHook = ''
    export ROOT_DIR=$(${pkgs.git}/bin/git rev-parse --show-toplevel)
    ${pkgs.cowsay}/bin/cowsay "Working on project root directory: ${rootDir}"
    cd "${rootDir}/tests/prism-test"
  '';

}
