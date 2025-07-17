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
    transactGenesis = writeShellApplication {
      name = "transactGenesis";
      text = ''
        mkdir -p transactions

        echo "Getting utxos"
        TX_IN="$(cardano-cli conway query utxo --address "$(cat ./testnet/utxo-keys/utxo1/utxo.addr)" | jq -r '. | keys[]')"

        echo "Drafting transaction"
        cardano-cli conway transaction build \
          --tx-in "$TX_IN" \
          --tx-out 'addr_test1qp83v2wq3z9mkcjj5ejlupgwt6tcly5mtmz36rpm8w4atvqd5jzpz23y8l4dwfd9l46fl2p86nmkkx5keewdevqxhlyslv99j3+100000000000' \
          --change-address addr_test1qp83v2wq3z9mkcjj5ejlupgwt6tcly5mtmz36rpm8w4atvqd5jzpz23y8l4dwfd9l46fl2p86nmkkx5keewdevqxhlyslv99j3 \
          --out-file transactions/tx.raw

        echo "Signing transaction"
        cardano-cli conway transaction sign \
          --tx-body-file transactions/tx.raw \
          --signing-key-file ./testnet/utxo-keys/utxo1/utxo.skey \
          --out-file transactions/tx.signed

        echo "Submitting transaction"
        cardano-cli conway transaction submit --tx-file transactions/tx.signed
      '';
    };
  };
  basePackages = with pkgs; [
    bash
    coreutils
    gawk
    gnugrep
    jq
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
      "CARDANO_CLI=${pkgs.cardano-cli}/bin/cardano-cli"
      "CARDANO_NODE=${pkgs.cardano-node}/bin/cardano-node"
    ];
    Entrypoint = [ ];
    Cmd = [ ];
    WorkingDir = "/node";
  };
}
