{
  pkgs,
  dockerTools,
  bash,
  coreutils,
  cardano-cli,
  cardano-node,
  cardano-testnet,
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
  contents = [
    bash
    coreutils
    cardano-cli
    cardano-node
    cardano-testnet
  ] ++ debugPackages;
  config = {
    Env = [ ];
    Entrypoint = [ ];
    Cmd = [ ];
    WorkingDir = "/node";
  };
}
