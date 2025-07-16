{
  pkgs,
  mkShell,
}:

let
  rootDir = "$ROOT_DIR";
in
mkShell {
  packages = with pkgs; [
    jq
    hurl
    cardano-node
    cardano-cli
    cardano-wallet
  ];

  shellHook = ''
    export ROOT_DIR=$(${pkgs.git}/bin/git rev-parse --show-toplevel)
    ${pkgs.cowsay}/bin/cowsay "Working on project root directory: ${rootDir}"
    cd "${rootDir}"
  '';
}
