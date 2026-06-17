{
  pkgs,
  config,
  ...
}:
{
  packages = with pkgs; [
    oxfmt
    nixfmt
    statix
    deadnix
    jq
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
