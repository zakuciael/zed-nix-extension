{ self, inputs, ... }:
{
  flake-file.inputs = {
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  perSystem =
    { system, pkgs, ... }:
    {
      imports = [
        "${inputs.nixpkgs}/nixos/modules/misc/nixpkgs.nix"
      ];
      nixpkgs = {
        hostPlatform = system;
        overlays = [
          (import inputs.rust-overlay)
        ];
      };

      _module.args.toolchain =
        (pkgs.rust-bin.fromRustupToolchainFile (self + /rust-toolchain.toml)).override
          {
            extensions = [
              "rust-src"
              "rust-analyzer"
              "clippy"
            ];
          };
    };
}

# {
#   inputs,
#   ...
# }:
# {
#   imports = [
#     inputs.rust-flake.flakeModules.nixpkgs
#     inputs.rust-flake.flakeModules.default
#   ];

#   flake-file.inputs = {
#     rust-flake = {
#       url = "github:juspay/rust-flake";
#       inputs.nixpkgs.follows = "nixpkgs";
#     };
#   };

#   perSystem =
#     { self', ... }:
#     {
#       rust-project = {
#         # See /crates/*/crate.nix for the crate-specific Nix configuration
#         crateNixFile = "crate.nix";
#       };

#       # packages = {
#       #   default = self'.packages.wf-companion-app;
#       # };
#     };
# }
