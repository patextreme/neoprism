{
  pkgs,
  dockerTools,
  bash,
  coreutils,
  cardano-cli,
  cardano-node,
  cardano-testnet,
  cardano-tracer,
}:

let
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
    cardano-testnet
    cardano-tracer
  ];
  config = {
    Env = [ ];
    Entrypoint = [ ];
    Cmd = [ ];
    WorkingDir = "/node";
  };
}
