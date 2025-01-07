{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    # rust-overlay = {
    #   url = "github:oxalica/rust-overlay";
    #   inputs.nixpkgs.follows = "nixpkgs";
    # };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      # rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        # pkgs = import nixpkgs {
        #   inherit system;
        #   overlays = [ (import rust-overlay) ];
        # };
        pkgs = nixpkgs.legacyPackages.${system};
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
        inherit (pkgs) lib stdenv;
        # craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default.override { });
        craneLib = crane.mkLib pkgs;
        sourceFilter =
          path: type: (lib.hasSuffix "/include.hpp" path) || (craneLib.filterCargoSources path type);

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
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
            # BINDGEN_EXTRA_CLANG_ARGS = "-std=c++20";
            # BINDGEN_EXTRA_CLANG_ARGS = lib.concatStringsSep " " [
            #   "-std=c++20"
            #   (builtins.readFile "${stdenv.cc}/nix-support/cc-cflags")
            #   (builtins.readFile "${stdenv.cc}/nix-support/libc-cflags")
            #   (builtins.readFile "${stdenv.cc}/nix-support/libc-crt1-cflags")
            #   (builtins.readFile "${stdenv.cc}/nix-support/libcxx-cxxflags")
            #   (builtins.readFile "${stdenv.cc}/nix-support/libcxx-ldflags")
            #   # "-idirafter ${pkgs.boost.lib}/include"
            #
            #   # (builtins.readFile "${stdenv.cc}/nix-support/cc-ldflags")
            #   # (builtins.readFile "${stdenv.cc}/nix-support/cc-cflags-before")
            #
            #   (lib.optionalString stdenv.cc.isClang "-idirafter ${stdenv.cc.cc}/lib/clang/${lib.getVersion stdenv.cc.cc}/include")
            #   (lib.optionalString stdenv.cc.isGNU "-isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc} -isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc}/${stdenv.hostPlatform.config} -idirafter ${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}/${lib.getVersion stdenv.cc.cc}/include -idirafter ${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}/14.2.1/include")
            # ];
            # RAGENIX_CLIB_PREFIX = "${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}";
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
                  # llvmPackages.libclang.lib
                  # llvmPackages.clang
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
            main.args // { buildPhaseCargoCommand = "cargo fmt --all -- --check"; }
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
