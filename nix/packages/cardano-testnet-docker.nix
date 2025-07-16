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
        # create configurations
        cardano-cli conway genesis create-cardano \
          --genesis-dir genesis \
          --gen-genesis-keys 1 \
          --gen-utxo-keys 1 \
          --start-time "$(date -u -d "now + 10 seconds" +%FT%Tz)" \
          --supply 1000000000 \
          --alonzo-template config/template/alonzo.json \
          --byron-template config/template/byron.json \
          --conway-template config/template/conway.json \
          --shelley-template config/template/shelley.json \
          --node-config-template config/template/config.json

        cardano-cli conway genesis create-testnet-data \
          --out-dir testnet \
          --spec-shelley genesis/shelley-genesis.json \
          --spec-alonzo genesis/alonzo-genesis.json \
          --spec-conway genesis/conway-genesis.json \
          --pools 1 
      '';
    };
    transactGenesis = writeShellApplication {
      name = "transactGenesis";
      text = ''
        TX_IN="$(cardano-cli conway query utxo --address "$(cat ./genesis/utxo-keys/shelley.000.addr)" | jq -r '. | keys[]')"
        cardano-cli conway transaction build \
          --tx-in "$TX_IN" \
          --tx-out 'addr_test1qp83v2wq3z9mkcjj5ejlupgwt6tcly5mtmz36rpm8w4atvqd5jzpz23y8l4dwfd9l46fl2p86nmkkx5keewdevqxhlyslv99j3+100000000000' \
          --change-address addr_test1qp83v2wq3z9mkcjj5ejlupgwt6tcly5mtmz36rpm8w4atvqd5jzpz23y8l4dwfd9l46fl2p86nmkkx5keewdevqxhlyslv99j3 \
          --out-file tx.raw
      '';
    };
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
    ];
    Entrypoint = [ ];
    Cmd = [ ];
    WorkingDir = "/node";
  };
}
