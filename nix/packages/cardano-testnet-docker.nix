{
  pkgs,
  dockerTools,
  writeShellApplication,
  bash,
  coreutils,
  cardano-cli,
  cardano-node,
}:

let
  initTestnet = writeShellApplication {
    name = "initTestnet";
    text = ''
      # create configurations
      cardano-cli legacy genesis create-cardano \
        --testnet-magic 42 \
        --genesis-dir genesis \
        --gen-genesis-keys 1 \
        --gen-utxo-keys 1 \
        --start-time "$(date -u -d "now + 10 seconds" +%FT%Tz)" \
        --supply 1000000000 \
        --conway-era \
        --alonzo-template config/template/alonzo.json \
        --byron-template config/template/byron.json \
        --conway-template config/template/conway.json \
        --shelley-template config/template/shelley.json \
        --node-config-template config/template/config.json

      cardano-cli address build \
        --payment-verification-key-file ./genesis/utxo-keys/shelley.000.vkey \
        --out-file ./genesis/utxo-keys/shelley.000.addr
    '';
  };

  debugPackages = with pkgs; [
    helix
    jq
    nettools
    yazi
  ];
in
dockerTools.buildLayeredImage {
  name = "cardano-testnet";
  tag = "latest";
  contents = debugPackages ++ [
    bash
    coreutils
    cardano-cli
    cardano-node
    initTestnet
  ];
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
