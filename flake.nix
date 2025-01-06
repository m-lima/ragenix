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
        lib = pkgs.lib;
        stdenv = pkgs.stdenv;
        craneLib = crane.mkLib pkgs;

        env = {
          CARGO_BUILD_RUSTFLAGS = "-C target-cpu=native -C prefer-dynamic=no";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = lib.concatStringsSep " " [
            (builtins.readFile "${stdenv.cc}/nix-support/libc-crt1-cflags")
            (builtins.readFile "${stdenv.cc}/nix-support/libc-cflags")
            (builtins.readFile "${stdenv.cc}/nix-support/cc-cflags")
            (builtins.readFile "${stdenv.cc}/nix-support/libcxx-cxxflags")
            (lib.optionalString stdenv.cc.isClang "-idirafter ${stdenv.cc.cc}/lib/clang/${lib.getVersion stdenv.cc.cc}/include")
            (lib.optionalString stdenv.cc.isGNU "-isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc} -isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc}/${stdenv.hostPlatform.config} -idirafter ${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}/${lib.getVersion stdenv.cc.cc}/include")
          ];
        };

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          nativeBuildInputs = with pkgs; [
            pkg-config
            llvmPackages.libclang.lib
          ];
          buildInputs = with pkgs; [ ] ++ lib.optionals stdenv.isDarwin [ libiconv ];
        } // env;

        cargoArtifacts = craneLib.buildDepsOnly (
          {
            preBuild = "cpp -v /dev/null -o /dev/null";
          }
          // commonArgs
        );

        ragenix = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });

        hack =
          {
            args,
            tools ? [ ],
          }:
          craneLib.mkCargoDerivation (
            commonArgs
            // {
              inherit cargoArtifacts;
              pnameSuffix = "-hack";
              buildPhaseCargoCommand = "cargo hack --feature-powerset --workspace ${args}";
              nativeBuildInputs = (commonArgs.nativeBuildInputs or [ ]) ++ [ pkgs.cargo-hack ] ++ tools;
            }
          );
      in
      {
        checks = {
          inherit ragenix;

          hackCheck = hack {
            args = "check";
          };
          hackCheckTests = hack {
            args = "check --tests";
          };
          hackCheckExamples = hack {
            args = "check --examples";
          };
          hackClippy = hack {
            args = "clippy";
            tools = [ pkgs.clippy ];
          };
          hackClippyTests = hack {
            args = "clippy --tests";
            tools = [ pkgs.clippy ];
          };
          hackClippyExamples = hack {
            args = "clippy --examples";
            tools = [ pkgs.clippy ];
          };
          hackTest = hack {
            args = "test";
          };
        };

        packages.default = ragenix;

        apps.default = flake-utils.lib.mkApp {
          drv = ragenix;
        };

        devShells.default = craneLib.devShell {
          inherit env;
          checks = self.checks.${system};
          packages = with pkgs; [
            cargo-hack
            libclang
          ];
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
