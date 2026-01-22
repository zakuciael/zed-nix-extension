{
  perSystem =
    {
      config,
      pkgs,
      toolchain,
      ...
    }:
    {
      devShells = {
        default = pkgs.mkShell {
          name = "zed-nix-extension";

          inputsFrom = [
            config.pre-commit.devShell
          ];
          buildInputs = [
            pkgs.libiconv
          ];
          packages = with pkgs; [
            toolchain
            nixfmt
            nixd
          ];

          shellHook = ''
            ${config.pre-commit.settings.shellHook}

            # For rust-analyzer 'hover' tooltips to work.
            export RUST_SRC_PATH="${toolchain}/lib/rustlib/src/rust/library";
          '';
        };
      };
    };
}
