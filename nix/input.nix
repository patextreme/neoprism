let sources = import ./sources.nix;
in rec {
  pkgs = import sources.nixpkgs {
    config = { allowUnfree = true; };
    overlays = [ (import sources.rust-overlay) ];
  };

  rust = pkgs.rust-bin.stable.latest.default.override {
    extensions = [ "rust-src" "rust-analyzer" ];
    targets = [ ];
  };

  oura = pkgs.rustPlatform.buildRustPackage {
    name = "oura";
    src = pkgs.fetchgit {
      url = "https://github.com/txpipe/oura.git";
      rev = "v1.8.1";
      sha256 = "HSVVrhwPPUeAHiIx/16r86pKQtCsNIQSVcTH92cLdNE=";
    };
    cargoHash = "sha256-a+j40vo/xzHx64pRdmN8gIDhXCi5xgrbWp9Bx15EXbU=";
    buildNoDefaultFeatures = true;
    buildFeatures = [ "logs" ];
  };
}
