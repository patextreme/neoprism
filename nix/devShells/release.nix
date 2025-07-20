{ pkgs, buildConfig }:

let
  rootDir = "$ROOT_DIR";
  rust = pkgs.rustUtils.rust;
  scripts = rec {
    inherit buildConfig;

    bumpVersion = pkgs.writeShellApplication {
      name = "bumpVersion";
      runtimeInputs = with pkgs; [ git-cliff jq ];
      text = ''
        cd "${rootDir}"
        NEW_VERSION=$(git-cliff --bump --context | jq -r .[0].version | sed s/^v//)
        ${setVersion}/bin/setVersion "$NEW_VERSION"
      '';
    };

    setVersion = pkgs.writeShellApplication {
      name = "setVersion";
      runtimeInputs = with pkgs; [ rust cargo-edit ];
      text = ''
        cd "${rootDir}"
        NEW_VERSION=$1
        echo "Setting new version to $NEW_VERSION"
        echo "$NEW_VERSION" > version
        cargo set-version "$NEW_VERSION"
        ${buildConfig}/bin/buildConfig
        git-cliff -t "$NEW_VERSION" > CHANGELOG.md
      '';
    };
  };
in
pkgs.mkShell {
  packages =
    with pkgs;
    [
      git
      nix
    ]
    ++ (builtins.attrValues scripts);

  shellHook = ''
    export ROOT_DIR=$(${pkgs.git}/bin/git rev-parse --show-toplevel)
    ${pkgs.cowsay}/bin/cowsay "Working on project root directory: ${rootDir}"
    cd "${rootDir}"
  '';
}
