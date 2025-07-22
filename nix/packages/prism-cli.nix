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
    version = "v0.1.0-M27";
    depsSha256 = "sha256-L8cGfwptKpQVxrMpXZ3HiAjWcHQrT5uHbpLCzd8vczw=";
    src = fetchFromGitHub {
      owner = "FabioPinheiro";
      repo = "scala-did";
      rev = version;
      hash = "sha256-gEnp7gHRixnA9lSNIL6wxvQm5WPTctqnghcEOX3zL5Y=";
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
