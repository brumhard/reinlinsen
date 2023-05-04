{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, utils, naersk, fenix }: utils.lib.eachDefaultSystem
    (system:
      let
        name = "rl";
        version = "latest";
        # https://discourse.nixos.org/t/using-nixpkgs-legacypackages-system-vs-import/17462/7
        pkgs = nixpkgs.legacyPackages.${system};
        naersk' = naersk.lib.${system};
      in
      with pkgs;
      rec {
        packages = {
          default = packages.${name};
          "${name}" = naersk'.buildPackage {
            inherit name version;
            src = ./.;
          };
        };

        apps = {
          default = apps.${name};
          "${name}" = utils.lib.mkApp {
            drv = packages.default;
            exePath = "/bin/${name}";
          };
        };

        devShell = mkShell {
          packages = [
            cargo-audit
            mask
          ];
        };
      }
    );
}
