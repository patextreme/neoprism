{
  dockerTools,
  ui-assets,
  neoprism-bin,
  version,
  tagSuffix ? "",
}:

dockerTools.buildLayeredImage {
  name = "neoprism";
  tag = "${version}${tagSuffix}";
  contents = [
    neoprism-bin
    ui-assets
  ];
  config = {
    Env = [ "RUST_LOG=info,oura=warn" ];
    Entrypoint = [ "/bin/indexer-node" ];
    Cmd = [
      "--assets"
      "/assets"
    ];
    WorkingDir = "/";
  };
}
