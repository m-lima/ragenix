{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
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
          path: type: (lib.hasSuffix "/ragenix.cc" path) || (craneLib.filterCargoSources path type);

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

        packages = {
          default = main.build;
          ragenix = main.build;
        };

        devShells.default = craneLib.devShell {
          env = main.env;
          checks = checks;
          packages = with pkgs; [ cargo-hack ];
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    )
    // {
      nixosModules =
        let
          module =
            {
              lib,
              config,
              pkgs,
              ...
            }:
            {
              options = {
                ragenix = {
                  pubKey = lib.mkOption {
                    type = lib.types.path;
                    description = "Path to key for decryption of the secret";
                    example = /home/user/.ssh/id_ed25519;
                  };
                };
              };

              config = {
                environment.systemPackages = [ self.packages.${pkgs.system}.ragenix ];
                nix.settings.plugin-files = [ "${self.packages.${pkgs.system}.ragenix}/lib/libragenix.so" ];
              };
            };
        in
        {
          default = module;
          ragenix = module;
        };
    };
}
