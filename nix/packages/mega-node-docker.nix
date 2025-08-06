{
  dockerTools,
  pkgsInternal,
}:

dockerTools.buildLayeredImage {
  name = "mega-node";
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
