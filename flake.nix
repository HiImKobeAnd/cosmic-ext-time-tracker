{
  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        deps = with pkgs; [
          rust-bin.stable.latest.default
          cargo-watch
          just
          libcosmicAppHook
        ];
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = deps;
            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath deps}";
          };
      }
    );
}
