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
        pkgs = nixpkgs.legacyPackages.${system}.pkgsLLVM;
        # (
        #     final: prev: {
        #       stdenv = nixpkgs.legacyPackages.${system}.pkgsLLVM;
        #       # stdenv = { };
        #     }
        #   );
        # pkgs = import nixpkgs {
        #   inherit system;
        #   overlays = [
        #     (final: prev: {
        #       stdenv = prev.clangStdenv;
        #       # stdenv = { };
        #     })
        #   ];
        # };
        # sysPkgs = nixpkgs.legacyPackages.${system};
        # pkgs = sysPkgs // {
        #   overlays = [
        #     (final: prev: {
        #       stdenv = prev.clangStdenv;
        #       # stdenv = { };
        #     })
        #   ];
        # };
        # pkgs = sysPkgs // {
        #   stdenv = sysPkgs.llvmPackages.libcxxStdenv;
        # };
        inherit (pkgs) lib;
        craneLib = crane.mkLib pkgs;

        commonEnv = {
          CARGO_BUILD_RUSTFLAGS = "-C target-cpu=native -C prefer-dynamic=no";
        };
        env = commonEnv // {
          # LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = "-std=c++20";
          # BINDGEN_EXTRA_CLANG_ARGS = lib.concatStringsSep " " [
          #   "-std=c++20"
          #   (builtins.readFile "${stdenv.cc}/nix-support/cc-cflags")
          #   (builtins.readFile "${stdenv.cc}/nix-support/libc-cflags")
          #   (builtins.readFile "${stdenv.cc}/nix-support/libc-crt1-cflags")
          #   (builtins.readFile "${stdenv.cc}/nix-support/libcxx-cxxflags")
          #   (builtins.readFile "${stdenv.cc}/nix-support/libcxx-ldflags")
          #
          #   # (builtins.readFile "${stdenv.cc}/nix-support/cc-ldflags")
          #   # (builtins.readFile "${stdenv.cc}/nix-support/cc-cflags-before")
          #
          #   (lib.optionalString stdenv.cc.isClang "-idirafter ${stdenv.cc.cc}/lib/clang/${lib.getVersion stdenv.cc.cc}/include")
          #   (lib.optionalString stdenv.cc.isGNU "-isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc} -isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc}/${stdenv.hostPlatform.config} -idirafter ${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}/${lib.getVersion stdenv.cc.cc}/include")
          # ];
          # RAGENIX_CLIB_PREFIX = "${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}";
        };

        sourceFilter =
          path: type: (lib.hasSuffix "/include.hpp" path) || (craneLib.filterCargoSources path type);

        commonArgs = commonEnv // {
          src = lib.cleanSourceWith {
            src = ./.;
            filter = sourceFilter;
            name = "source";
          };
          strictDeps = true;

          nativeBuildInputs = [ ];
          buildInputs = with pkgs; [ ] ++ lib.optionals stdenv.isDarwin [ libiconv ];
        };
        args =
          commonArgs
          // env
          // {
            nativeBuildInputs =
              with pkgs;
              commonArgs.nativeBuildInputs
              ++ [
                pkg-config
                # llvmPackages.libclang.lib
                # llvmPackages.clang
              ];
            buildInputs =
              with pkgs;
              commonArgs.buildInputs
              ++ [
                nix
                boost
              ];
          };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        ragenix = craneLib.buildPackage (args // { inherit cargoArtifacts; });

        hack =
          {
            extraArgs,
            tools ? [ ],
          }:
          craneLib.mkCargoDerivation (
            args
            // {
              inherit cargoArtifacts;
              pnameSuffix = "-hack";
              buildPhaseCargoCommand = "cargo hack --feature-powerset --workspace ${extraArgs}";
              nativeBuildInputs = (commonArgs.nativeBuildInputs or [ ]) ++ [ pkgs.cargo-hack ] ++ tools;
            }
          );
      in
      {
        checks = {
          inherit ragenix;

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
            extraArgs = "clippy";
            tools = [ pkgs.clippy ];
          };
          hackClippyTests = hack {
            extraArgs = "clippy --tests";
            tools = [ pkgs.clippy ];
          };
          hackClippyExamples = hack {
            extraArgs = "clippy --examples";
            tools = [ pkgs.clippy ];
          };
          hackTest = hack {
            extraArgs = "test";
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
            # libclang
          ];
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
