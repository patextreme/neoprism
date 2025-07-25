{
  curl,
  dockerTools,
  neoprism-bin,
  tagSuffix ? "",
  neoprism-ui-assets,
  version,
}:

dockerTools.buildLayeredImage {
  name = "identus-neoprism";
  tag = "${version}${tagSuffix}";
  contents = [
    curl
    neoprism-bin
    neoprism-ui-assets
  ];
  config = {
    Env = [
      "RUST_LOG=info,oura=warn"
      "NPRISM_ASSETS_PATH=/assets"
    ];
    Entrypoint = [ "/bin/neoprism-node" ];
    Cmd = [ ];
    WorkingDir = "/";
  };
}
