{
  dockerTools,
  bash,
  coreutils,
  cardano-node,
  cardano-cli,
}:

dockerTools.buildLayeredImage {
  name = "cardano-testnet";
  tag = "latest";
  contents = [
    bash
    coreutils
    cardano-node
    cardano-cli
  ];
  config = {
    Env = [ ];
    Entrypoint = [ ];
    Cmd = [ ];
    WorkingDir = "/";
  };
}
