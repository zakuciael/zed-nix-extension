{
  inputs = {
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    zed-extensions = {
      url = "github:DuskSystems/nix-zed-extensions";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } (
      { self, ... }: {
        debug = true;
        systems = [ "x86_64-linux" ];

        perSystem =
          {
            pkgs,
            lib,
            self',
            system,
            ...
          }:
          {
            _module.args.pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [
                inputs.zed-extensions.overlays.default
              ];
            };

            packages =
              let
                inherit (lib) importTOML;
                inherit (pkgs) fetchFromGitHub buildZedRustExtension buildZedGrammar;
                extensionToml = importTOML (self + "/extension.toml");

                grammar = extensionToml.grammars.nix;

                repoUrlMatch = builtins.match "https://github\.com/([^/]+)/([^/]+)(/.*)?$" grammar.repository;

                repoUrlParts =
                  if repoUrlMatch != null then
                    repoUrlMatch
                  else
                    throw ''
                      Invalid GitHub URL format: ${grammar.repository}
                      Expected format: https://github.com/owner/repo
                    '';
              in
              {
                zed-nix-grammar = buildZedGrammar (attrs: {
                  name = "nix";
                  version = extensionToml.grammars.nix.commit;

                  src = fetchFromGitHub {
                    owner = lib.elemAt repoUrlParts 0;
                    repo = lib.elemAt repoUrlParts 1;
                    rev = attrs.version;
                    hash = "sha256-KQ00kJo350Xhj2pFaaYDcgXvv1CxunnhWIBZth2e5es=";
                  };
                });

                zed-nix-extension = buildZedRustExtension {
                  name = "nix";
                  inherit (extensionToml) version;

                  src = self;
                  cargoLock.lockFile = self + "/Cargo.lock";

                  grammars = [
                    self'.packages.zed-nix-grammar
                  ];
                };
              };
          };
      }
    );
}
