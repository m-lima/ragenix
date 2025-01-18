{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      crane,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs) lib stdenv;
        craneLib = crane.mkLib pkgs;
        sourceFilter =
          path: type: (lib.hasSuffix "/include.h" path) || (craneLib.filterCargoSources path type);

        deps = {
          env = {
            CARGO_BUILD_RUSTFLAGS = "-C target-cpu=native -C prefer-dynamic=no";
          };

          args = deps.env // {
            src = lib.cleanSourceWith {
              src = ./.;
              filter = sourceFilter;
              name = "source";
            };
            strictDeps = true;

            nativeBuildInputs = [ ];
            buildInputs = with pkgs; [ ] ++ lib.optionals stdenv.isDarwin [ libiconv ];
          };

          build = craneLib.buildDepsOnly deps.args;
        };

        main = {
          env = deps.env // {
            NIX_OUTPATH_USED_AS_RANDOM_SEED = "ragenixout";
          };

          args =
            deps.args
            // main.env
            // {
              cargoArtifacts = deps.build;
              nativeBuildInputs =
                with pkgs;
                deps.args.nativeBuildInputs
                ++ [
                  pkg-config
                  rustPlatform.bindgenHook
                ];
              buildInputs =
                with pkgs;
                deps.args.buildInputs
                ++ [
                  nix
                  boost
                ];
            };

          build = craneLib.buildPackage main.args;
        };

        hack =
          {
            extraArgs,
            tools ? [ ],
          }:
          craneLib.mkCargoDerivation (
            main.args
            // {
              pnameSuffix = "-hack";
              buildPhaseCargoCommand = "cargo hack --feature-powerset --workspace ${extraArgs}";
              nativeBuildInputs = (main.args.nativeBuildInputs or [ ]) ++ [ pkgs.cargo-hack ] ++ tools;
            }
          );

        checks = {
          ragenix = main.build;

          fmt = craneLib.mkCargoDerivation (
            main.args
            // {
              buildPhaseCargoCommand = "cargo fmt --all -- --check";
              nativeBuildInputs = (main.args.nativeBuildInputs or [ ]) ++ [ pkgs.rustfmt ];
            }
          );
          hackCheck = hack {
            extraArgs = "check";
          };
          hackCheckTests = hack {
            extraArgs = "check --tests";
          };
          hackCheckExamples = hack {
            extraArgs = "check --examples";
          };
          hackClippy = hack {
            extraArgs = "clippy -- -D warnings -W clippy::pedantic";
            tools = [ pkgs.clippy ];
          };
          hackClippyTests = hack {
            extraArgs = "clippy --tests -- -D warnings -W clippy::pedantic";
            tools = [ pkgs.clippy ];
          };
          hackClippyExamples = hack {
            extraArgs = "clippy --examples -- -D warnings -W clippy::pedantic";
            tools = [ pkgs.clippy ];
          };
          hackTest = hack {
            extraArgs = "test";
          };
        };
      in
      {
        checks = checks;

        packages.default = main.build;

        apps.default = flake-utils.lib.mkApp {
          drv = main.build;
        };

        devShells.default = craneLib.devShell {
          env = main.env;
          checks = checks;
          packages = with pkgs; [ cargo-hack ];
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
