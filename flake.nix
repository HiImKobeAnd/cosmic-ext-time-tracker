{
  description = "";

  inputs = {
    nixpkgs.url = "nixpkgs";
  };

  outputs =
    { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;

      systems = [
        "x86_64-linux"
      ];
      forEachSystem =
        perSystem:
        lib.genAttrs systems (
          system:
          let
            pkgs = nixpkgs.legacyPackages.${system};
          in
          perSystem { inherit pkgs system; }
        );
      version = "0.0.1";
    in
    {
      overlays.default = final: prev: {
        cosmic-ext-time-tracker = final.callPackage ./nix/cosmic-ext-time-tracker-package.nix {
          inherit version;
        };
      };
      packages = forEachSystem (
        { pkgs, ... }:
        {
          default = pkgs.callPackage ./nix/cosmic-ext-time-tracker-package.nix { inherit version; };
        }
      );
      devShells = forEachSystem (
        { pkgs, system }:
        {
          default = pkgs.callPackage ./nix/devshell.nix {
            cosmic-ext-time-tracker = self.packages.${system}.default;
          };
        }
      );
      apps = forEachSystem (

        { pkgs, system }:
        {
          default = {
            type = "app";
            program = lib.getExe self.packages.${system}.default;
          };
        }
      );
    };
}
