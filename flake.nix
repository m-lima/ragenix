{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixref.url = "github:NixOS/nixpkgs/nixos-24.11";
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
      nixref,
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
      buildInputs = pkgs: [
        nixref.legacyPackages.${pkgs.system}.nix
        pkgs.boost
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
