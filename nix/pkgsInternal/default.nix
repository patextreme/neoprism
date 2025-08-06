{ pkgs }:

{
  scala-did = pkgs.callPackage ./scala-did { };
  prism-node = pkgs.callPackage ./prism-node { };
}
