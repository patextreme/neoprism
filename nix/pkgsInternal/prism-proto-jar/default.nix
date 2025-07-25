{
  mkSbtDerivation,
  fetchFromGitHub,
  protobuf,
  git,
  cacert,
  lib,
  stdenv,
}:

mkSbtDerivation rec {
  pname = "prism-proto-jar";
  version = "update-proto";
  depsSha256 = "sha256-wTfPTgEs6E8YxAfefE1vZ6wOAnTPAFB9kqA6KS1SgOs=";
  src = fetchFromGitHub {
    owner = "hyperledger-identus";
    repo = "cloud-agent";
    rev = version;
    hash = "sha256-OpvtTzLs4Gz/GefXr5jOIHmaI6J/wqpPb34DSVs4JQI=";
  };
  patches = [
    ./source.patch
  ];

  nativeBuildInputs = [
    git
    protobuf
    cacert
  ];

  LANG = "C.utf8";

  depsWarmupCommand = ''
    export LD_LIBRARY_PATH=${
      lib.makeLibraryPath [
        stdenv.cc.cc
      ]
    }
    sbt prismNodeClient/compile
  '';

  buildPhase = ''
    export LD_LIBRARY_PATH=${
      lib.makeLibraryPath [
        stdenv.cc.cc
      ]
    }
    sbt prismNodeClient/packageBin
    sbt prismNodeClient/packageSrc
  '';

  installPhase = ''
    mkdir -p $out/jars
    cp -r prism-node/client/scala-client/target/scala-3.3.5/*.jar $out/jars
  '';
}
