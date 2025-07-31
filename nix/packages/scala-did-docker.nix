{
  dockerTools,
  pkgsInternal,
}:

dockerTools.buildLayeredImage {
  name = "scala-did";
  tag = "latest";
  contents = [
    pkgsInternal.scala-did-node
  ];
  config = {
    Env = [ ];
    Entrypoint = [ "/bin/scala-did-node" ];
    Cmd = [ ];
    WorkingDir = "/";
  };
}
