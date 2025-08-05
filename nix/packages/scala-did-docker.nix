{
  dockerTools,
  pkgsInternal,
}:

dockerTools.buildLayeredImage {
  name = "prism-node-all";
  tag = "latest";
  contents = [
    pkgsInternal.scala-did
    pkgsInternal.prism-node
  ];
  config = {
    Env = [ ];
    Entrypoint = [ ];
    Cmd = [ ];
    WorkingDir = "/";
  };
}
