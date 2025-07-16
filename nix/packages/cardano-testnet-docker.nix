{
  pkgs,
  dockerTools,
  writeShellApplication,
}:

let
  scripts = {
    initTestnet = writeShellApplication {
      name = "initTestnet";
      text = ''
        cardano-testnet cardano \
          --conway-era \
          --testnet-magic "$CARDANO_NODE_NETWORK_ID" \
          --output-dir testnet
      '';
    };
    # transactGenesis = writeShellApplication {
    #   name = "transactGenesis";
    #   text = ''
    #     TX_IN="$(cardano-cli conway query utxo --address "$(cat ./genesis/utxo-keys/shelley.000.addr)" | jq -r '. | keys[]')"
    #     cardano-cli conway transaction build \
    #       --tx-in "$TX_IN" \
    #       --tx-out 'addr_test1qp83v2wq3z9mkcjj5ejlupgwt6tcly5mtmz36rpm8w4atvqd5jzpz23y8l4dwfd9l46fl2p86nmkkx5keewdevqxhlyslv99j3+100000000000' \
    #       --change-address addr_test1qp83v2wq3z9mkcjj5ejlupgwt6tcly5mtmz36rpm8w4atvqd5jzpz23y8l4dwfd9l46fl2p86nmkkx5keewdevqxhlyslv99j3 \
    #       --out-file tx.raw
    #   '';
    # };
  };
  basePackages = with pkgs; [
    bash
    coreutils
    gawk
    gnugrep
    helix
    jq
    nettools
    yazi
  ];
  cardanoPackages = with pkgs; [
    cardano-node
    cardano-cli
    cardano-testnet
  ];
in
dockerTools.buildLayeredImage {
  name = "cardano-testnet";
  tag = "latest";
  contents = basePackages ++ cardanoPackages ++ (builtins.attrValues scripts);
  config = {
    Env = [
      "VISUAL=hx"
      "EDITOR=hx"
      "CARDANO_CLI=${pkgs.cardano-cli}/bin/cardano-cli"
      "CARDANO_NODE=${pkgs.cardano-node}/bin/cardano-node"
    ];
    Entrypoint = [ ];
    Cmd = [ ];
    WorkingDir = "/node";
  };
}
