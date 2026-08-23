{
  pkgs,
  config,
  ...
}:
let
  rustWorkspace =
    let
      inherit (config.languages.rust) toolchainPackage;

      crate2nix = config.lib.getInput {
        name = "crate2nix";
        url = "github:nix-community/crate2nix";
        attribute = "languages.rust.import";
        follows = [ "nixpkgs" ];
      };

      crate2nixTools = pkgs.callPackage "${crate2nix}/tools.nix" { };
      path = ./.;

      packageName =
        let
          cargoToml =
            if builtins.pathExists (path + "/Cargo.toml") then
              fromTOML (builtins.readFile (path + "/Cargo.toml"))
            else
              { };
        in
        cargoToml.package.name or (baseNameOf (toString path));

      cargoNix =
        pkgs.callPackage
          (crate2nixTools.generatedCargoNix {
            name = packageName;
            src = path;
          })
          {
            buildRustCrateForPkgs =
              _:
              pkgs.buildRustCrate.override {
                rustc = toolchainPackage;
                cargo = toolchainPackage;
              };
          };
    in
    cargoNix.workspaceMembers;
in
{
  packages = with pkgs; [
    oxfmt
    nixfmt
    statix
    deadnix
    jq

    rustWorkspace."statix-ls".build
    rustWorkspace."deadnix-ls".build
  ];

  claude.code = {
    enable = true;

    mcpServers.devenv = {
      type = "stdio";
      command = "devenv";
      args = [ "mcp" ];
      env = {
        DEVENV_ROOT = config.devenv.root;
      };
    };

    hooks.notify = {
      enable = true;
      name = "Send desktop notification when Claude needs attention";
      hookType = "Notification";
      command = "notify-send 'Claude Code' 'Needs attention'";
    };
  };

  languages = {
    nix = {
      enable = true;
      lsp = {
        enable = true;
        package = pkgs.nixd;
      };
    };
    rust = {
      enable = true;
      lsp.enable = true;
    };
  };

  git-hooks.hooks = {
    rustfmt.enable = true;
    clippy.enable = true;
    nixfmt.enable = true;
    oxfmt.enable = true;
  };

  tasks = {
    "ext:build" = {
      exec = ''
        jq -n --arg result_path "$(nix build .#zed-nix-extension --print-out-paths --no-link)" '{ "result_path": $result_path }' > $DEVENV_TASK_OUTPUT_FILE
      '';
    };
    "ext:link" = {
      after = [ "ext:build@succeeded" ];
      exec = ''
        ln -sf "$(echo $DEVENV_TASKS_OUTPUTS | jq -r '."ext:build".result_path')/share/zed/extensions/nix" ~/.local/share/zed/extensions/installed/
      '';
    };
  };
}
