{
  lib,
  mkSbtDerivation,
  writeShellApplication,
  fetchFromGitHub,
  stdenv,
  cacert,
  protobuf,
  temurin-jre-bin,
  symlinkJoin,
}:

let
  jars = mkSbtDerivation rec {
    pname = "prism-node";
    version = "v2.6.0";
    depsSha256 = "sha256-zvo+0rH1K6HXauKoAAGteOhW2T+9PHQsQUKDzmjxPcI=";
    src = fetchFromGitHub {
      owner = "input-output-hk";
      repo = "atala-prism";
      rev = version;
      hash = "sha256-3k8Fznmm5BKDvZzY9ZBtiQhzWppC8/pxWVJtQruWQ8A=";
    };
    patches = [ ./node_impl.patch ];

    nativeBuildInputs = [
      cacert
      protobuf
    ];

    LANG = "C.utf8";
    SSL_CERT_FILE = "${cacert}/etc/ssl/certs/ca-bundle.crt";

    depsWarmupCommand = ''
      export GITHUB_TOKEN=""
      export LD_LIBRARY_PATH=${
        lib.makeLibraryPath [
          stdenv.cc.cc
        ]
      }
      sbt compile
    '';

    buildPhase = ''
      export GITHUB_TOKEN=""
      export LD_LIBRARY_PATH=${
        lib.makeLibraryPath [
          stdenv.cc.cc
        ]
      }
      sbt assembly
    '';

    installPhase = ''
      mkdir -p $out/jars
      cp -r target/scala-2.13/node-assembly-2.6.0-SNAPSHOT.jar $out/jars/prism-node.jar
    '';
  };

  prism-node = writeShellApplication {
    name = "prism-node";
    runtimeInputs = [ temurin-jre-bin ];
    text = ''
      java -jar ${jars}/jars/prism-node.jar "$@"
    '';
  };
in
symlinkJoin {
  name = "prism-node";
  paths = [
    prism-node
    jars
  ];
}
