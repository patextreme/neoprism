{
  mkSbtDerivation,
  fetchFromGitHub,
  protobuf,
  git,
  cacert,
  lib,
  stdenv
}:

mkSbtDerivation rec {
  pname = "prism-node-client";
  version = "update-proto";
  depsSha256 = "sha256-hMDL+/r50NJlaE2O2cz0JlC7MA65BzxFpBqR/CXf67s=";
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
    sbt shared/compile
    sbt sharedCrypto/compile
    sbt prismNodeClient/compile
  '';

  buildPhase = ''
    export LD_LIBRARY_PATH=${
      lib.makeLibraryPath [
        stdenv.cc.cc
      ]
    }
    sbt shared/packageBin
    sbt shared/packageSrc
    sbt sharedCrypto/packageBin
    sbt sharedCrypto/packageSrc
    sbt prismNodeClient/packageBin
    sbt prismNodeClient/packageSrc
  '';

  installPhase = ''
    mkdir -p $out
    cp -r prism-node/client/scala-client/target/scala-3.3.5/*.jar $out/
    cp -r shared/core/target/*.jar $out/
    cp -r shared/crypto/target/*.jar $out/
  '';
}
