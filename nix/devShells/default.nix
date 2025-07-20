{ pkgs }:

let
  rootDir = "$ROOT_DIR";
  buildConfig = pkgs.writeShellApplication {
    name = "buildConfig";
    runtimeInputs = with pkgs; [ dhall-json ];
    text = ''
      cd "${rootDir}/docker/.config"
      dhall-to-yaml <<< "(./main.dhall).mainnet-relay" > "${rootDir}/docker/mainnet-relay/compose.yml"
      dhall-to-yaml <<< "(./main.dhall).mainnet-dbsync" > "${rootDir}/docker/mainnet-dbsync/compose.yml"
      dhall-to-yaml <<< "(./main.dhall).testnet-local" > "${rootDir}/docker/testnet-local/compose.yml"
    '';
  };
in
{
  default = import ./neoprism.nix { inherit pkgs buildConfig; };
  release = import ./release.nix { inherit pkgs buildConfig; };
  cardano = import ./cardano.nix { inherit pkgs; };
}
