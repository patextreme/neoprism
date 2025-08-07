{
  dockerTools,
  pkgsInternal,
}:

dockerTools.buildLayeredImage {
  name = "prism-node-fastsync";
  tag = "latest";
  contents = [
    # pkgsInternal.scala-did
    pkgsInternal.prism-node
  ];
  config = {
    Env = [ ];
    Entrypoint = [ "/bin/prism-node" ];
    Cmd = [ ];
    WorkingDir = "/";
  };
}
