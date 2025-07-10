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
    Env = [
      "RUST_LOG=info,oura=warn"
      "NPRISM_ASSETS_PATH=/assets"
    ];
    Entrypoint = [ "/bin/indexer-node" ];
    Cmd = [ ];
    WorkingDir = "/";
  };
}
