{ pkgs }:

let
  rootDir = "$ROOT_DIR";
  scripts = {
    createPrismKeys = pkgs.writeShellApplication {
      name = "createPrismKeys";
      text = ''
        cd "${rootDir}"
        rm -rf .cardano-prism-config.json
        prism-cli config-file --create
        prism-cli key --label master-0 --derivation-path "m/551'/21325'/0'/0'/0'"
        prism-cli key --label master-1 --derivation-path "m/551'/21325'/0'/0'/1'"
        prism-cli key --label master-2 --derivation-path "m/551'/21325'/0'/0'/2'"
        prism-cli key --label vdr-0 --derivation-path "m/551'/21325'/0'/8'/0'"
        prism-cli key --label vdr-1 --derivation-path "m/551'/21325'/0'/8'/1'"
        prism-cli key --label vdr-2 --derivation-path "m/551'/21325'/0'/8'/2'"
      '';
    };
  };
in
pkgs.mkShell {
  packages =
    with pkgs;
    [
      nix
      jq
      hurl
      cardano-node
      cardano-cli
      cardano-wallet
      cardano-testnet
      cardano-db-sync
      prism-cli
    ]
    ++ (builtins.attrValues scripts);

  shellHook = ''
    export ROOT_DIR=$(${pkgs.git}/bin/git rev-parse --show-toplevel)
    ${pkgs.cowsay}/bin/cowsay "Working on project root directory: ${rootDir}"
    cd "${rootDir}"
  '';

  CARDANO_CLI = "${pkgs.cardano-cli}/bin/cardano-cli";
  CARDANO_NODE = "${pkgs.cardano-node}/bin/cardano-node";
  PRISM_HOME = ".";
}
