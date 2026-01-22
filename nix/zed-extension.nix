{
  self,
  lib,
  inputs,
  ...
}:
{
  flake-file.inputs = {
    zed-extensions = {
      url = "github:DuskSystems/nix-zed-extensions";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nixpkgs-unstable.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
    };
  };

  perSystem =
    { self', pkgs, ... }:
    {
      nixpkgs.overlays = [
        inputs.zed-extensions.overlays.default
      ];

      packages =
        let
          inherit (lib) importTOML;
          inherit (pkgs) fetchFromGitHub buildZedRustExtension buildZedGrammar;
          extensionToml = importTOML (self + "/extension.toml");
        in
        {
          zed-nix-grammar = buildZedGrammar (attrs: {
            name = "nix";
            version = extensionToml.grammars.nix.commit;

            src = fetchFromGitHub {
              owner = "nix-community";
              repo = "tree-sitter-nix";
              rev = attrs.version;
              hash = "sha256-VNOPzeyhh/0jHzK0bwEX1kwSIUGoSlCXGhgjHbYvWKk=";
            };
          });
          zed-nix-extension = buildZedRustExtension {
            name = "nix";
            version = extensionToml.version;

            src = "${self}";
            cargoLock = {
              lockFile = self + "/Cargo.lock";
            };

            grammars = [
              self'.packages.zed-nix-grammar
            ];
          };
        };
    };
}
