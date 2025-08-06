{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    helper.url = "github:m-lima/nix-template";
  };

  outputs =
    {
      self,
      flake-utils,
      helper,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      (helper.lib.rust.helper inputs system ./. {
        allowFilesets = [ ./ragenix.cc ];
        lockRandomSeed = true;
        binary = false;
        nativeBuildInputs = pkgs: [
          pkgs.pkg-config
          pkgs.rustPlatform.bindgenHook
        ];
        buildInputs = pkgs: [
          pkgs.nix
          pkgs.boost
        ];
        formatters = {
          clang-format.enable = true;
        };
      }).outputs
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
                  key = lib.mkOption {
                    type = lib.types.path;
                    description = "Path to key for decryption of the secret";
                    example = /home/user/.ssh/id_ed25519;
                  };
                };
              };

              config =
                let
                  ragenix = self.packages.${pkgs.system}.default;
                in
                {
                  environment.systemPackages = [ ragenix ];
                  nix.settings.plugin-files = [ "${ragenix}/lib" ];
                };
            };
        in
        {
          default = module;
          ragenix = module;
        };
    };
}
