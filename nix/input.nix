let sources = import ./sources.nix;
in rec {
  pkgs = import sources.nixpkgs {
    config = { allowUnfree = true; };
    overlays = [ (import sources.nixpkgs-mozilla) ];
  };

  rust = pkgs.latest.rustChannels.stable.rust;

  rustPlatform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
}
