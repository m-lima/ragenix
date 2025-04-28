{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    rust-helper.url = "github:m-lima/nix-template";
  };

  outputs =
    {
      self,
      rust-helper,
      ...
    }@inputs:
    (rust-helper.lib.rust.helper inputs {
      allowFilesets = [ ./ragenix.cc ];
      lockRandomSeed = true;
      binary = false;
      fmts = [ "clang-format" ];
      nativeBuildInputs =
        pkgs: with pkgs; [
          pkg-config
          rustPlatform.bindgenHook
        ];
      buildInputs =
        pkgs: with pkgs; [
          nix
          boost
        ];
    } ./. "ragenix")
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

              config = {
                environment.systemPackages = [ self.packages.${pkgs.system}.ragenix ];
                nix.settings.plugin-files = builtins.attrNames (builtins.readDir "${self.packages.${pkgs.system}.ragenix}/lib");
              };
            };
        in
        {
          default = module;
          ragenix = module;
        };
    };
}
