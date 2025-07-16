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
    cardano-testnet
  ];

  shellHook = ''
    export ROOT_DIR=$(${pkgs.git}/bin/git rev-parse --show-toplevel)
    ${pkgs.cowsay}/bin/cowsay "Working on project root directory: ${rootDir}"
    cd "${rootDir}"
  '';

  CARDANO_CLI = "${pkgs.cardano-cli}/bin/cardano-cli";
  CARDANO_NODE = "${pkgs.cardano-node}/bin/cardano-node";
}
