{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    fenix,
  }:
    utils.lib.eachDefaultSystem
    (
      system: let
        name = "rl";
        version = "latest";
        # https://discourse.nixos.org/t/using-nixpkgs-legacypackages-system-vs-import/17462/7
        pkgs = nixpkgs.legacyPackages.${system};
        toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-3jVIIf5XPnUU1CRaTyAiO0XHVbJl12MSx3eucTXCjtE=";
        };
        naersk' = naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        };
      in
        with pkgs; rec {
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

          # should not include c toolchain but use host toolchain.
          # this seems to be required to cross compile x86_64-apple-darwin on M1
          # https://github.com/NixOS/nixpkgs/commit/9b3091a94cad63ebd0bd7aafbcfed7c133ef899d
          devShell = mkShellNoCC {
            packages = [
              rustup
              cargo-audit
              cargo-outdated
              cargo-cross
              cargo-edit

              mask
              yq-go
              ripgrep
              fd
              goreleaser
              svu
              commitlint
              syft
              cosign
            ];

            # https://github.com/openebs/mayastor-control-plane/blob/develop/shell.nix
            NODE_PATH = "${nodePackages."@commitlint/config-conventional"}/lib/node_modules";
            # see https://github.com/cross-rs/cross/issues/1241
            CROSS_CONTAINER_OPTS = "--platform linux/amd64";
          };
        }
    );
}
