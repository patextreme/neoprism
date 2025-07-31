{
  lib,
  mkSbtDerivation,
  writeShellApplication,
  fetchFromGitHub,
  stdenv,
  yarn,
  cacert,
  protobuf,
  jdk21,
}:

let
  cliJar = mkSbtDerivation rec {
    pname = "prism-cli";
    version = "v0.1.0-M28";
    depsSha256 = "sha256-wGuiyZGV6/yraYDig1loSK9J6peTDXWWe5iy3pLyNmQ=";
    src = fetchFromGitHub {
      owner = "FabioPinheiro";
      repo = "scala-did";
      rev = version;
      hash = "sha256-NMeiJrxOsa0F9oJqNm85DGtfsstunwWwJUrDZImdJ8w=";
    };

    nativeBuildInputs = [
      yarn
      cacert
      protobuf
    ];

    LANG = "C.utf8";
    SSL_CERT_FILE = "${cacert}/etc/ssl/certs/ca-bundle.crt";

    depsWarmupCommand = ''
      export LD_LIBRARY_PATH=${
        lib.makeLibraryPath [
          stdenv.cc.cc
        ]
      }
      sbt didResolverPrismJVM/compile
    '';

    buildPhase = ''
      export LD_LIBRARY_PATH=${
        lib.makeLibraryPath [
          stdenv.cc.cc
        ]
      }
      sbt didResolverPrismJVM/assembly
    '';

    installPhase = ''
      cp -r did-method-prism/jvm/target/scala-3.3.6/cardano-prism.jar $out
    '';
  };
in
writeShellApplication {
  name = "prism-cli";
  runtimeInputs = [ jdk21 ];
  text = ''
    if [ -z "''${PRISM_HOME:-}" ]; then
      echo "Error: PRISM_HOME cannot be empty."
      exit 1
    fi
    java -Duser.home="$PRISM_HOME" -jar ${cliJar} "$@"
  '';
}
