{
  pkgs,
  dockerTools,
  bash,
  coreutils,
  cardano-cli,
  cardano-node,
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
  ];
  config = {
    Env = [ ];
    Entrypoint = [ ];
    Cmd = [ ];
    WorkingDir = "/node";
  };
}
