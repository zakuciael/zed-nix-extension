# Nix

Nix language support in Zed

## Configuration

Various options can be configured via [Zed `settings.json`](https://zed.dev/docs/configuring-zed#settings-files) files.

### Configure Nixd

```json
{
  "lsp": {
    "nixd": {
      "settings": {
        "diagnostic": {
          "suppress": [ "sema-extra-with" ]
        }
      }
    }
  }
}
```

See: [Nixd LSP Configuration Docs](https://github.com/nix-community/nixd/blob/main/nixd/docs/configuration.md) for more options.

### Configure Nil

```json
{
  "lsp": {
    "nil": {
      "settings": {
         "diagnostics": {
          "ignored": [ "unused_binding" ]
        }
      }
    }
  }
}
```

See: [Nil LSP Configuration Docs](https://github.com/oxalica/nil/blob/main/docs/configuration.md) for more options.

### Configure statix

`statix` is a built-in language server that runs [statix](https://github.com/nerdypepper/statix) (Nix anti-pattern linter) and publishes its findings as inline diagnostics with quick-fix code actions.

The `statix-ls` binary must be available on your `PATH` (e.g. installed via nixpkgs), or it will be downloaded automatically from GitHub releases on first use.

> **Note:** statix must be compiled with `--all-features` for JSON output support. The `statix` package in nixpkgs includes this by default.

```json
{
  "lsp": {
    "statix": {
      "settings": {
        "binary": "/custom/path/to/statix-ls",
        "config": "/path/to/statix.toml"
      }
    }
  }
}
```

All fields are optional. `binary` overrides the `statix-ls` executable path; omitting it uses PATH then automatic download. `config` passes `--config` to statix; omitting it uses statix's native auto-discovery.

#### Disable statix

```json
{
  "languages": {
    "Nix": {
      "language_servers": ["nixd", "nil", "!statix"]
    }
  }
}
```

### Configure deadnix

`deadnix` is a built-in language server that runs [deadnix](https://github.com/astro/deadnix) (unused binding finder) and publishes its findings as inline diagnostics.

The `deadnix-ls` binary must be available on your `PATH` (e.g. installed via nixpkgs), or it will be downloaded automatically from GitHub releases on first use.

```json
{
  "lsp": {
    "deadnix": {
      "settings": {
        "binary": "/custom/path/to/deadnix-ls",
        "no_lambda_arg": false,
        "no_lambda_pattern_names": false,
        "no_underscore": false
      }
    }
  }
}
```

All fields are optional. `binary` overrides the `deadnix-ls` executable path. The `no_*` flags correspond to the matching deadnix CLI flags.

#### Disable deadnix

```json
{
  "languages": {
    "Nix": {
      "language_servers": ["nixd", "nil", "!deadnix"]
    }
  }
}
```

### Only use Nixd

```json
{
  "languages": {
    "Nix": {
      "language_servers": ["nixd", "!nil"]
    }
  }
}
```

### Only use Nil

```json
{
  "languages": {
    "Nix": {
      "language_servers": ["nil", "!nixd"]
    }
  }
}
```

## Runnable tasks

The extension detects flake output bindings (`packages`, `checks`, `devShells`, `apps`, `formatter`) and shows run buttons in the gutter. Clicking a button opens a task picker with relevant actions:

| Output      | Actions                                                   |
| ----------- | --------------------------------------------------------- |
| `packages`  | nix build, nix run, nix build --debugger                  |
| `checks`    | nix check, nix flake check (all), nix check --debugger    |
| `devShells` | nix develop, nix develop (impure), nix develop --debugger |
| `apps`      | nix run, nix run --debugger                               |
| `formatter` | nix fmt, nix fmt --check                                  |

Both 2-level (`packages.default`) and 3-level (`packages.x86_64-linux.default`) attrpath patterns are supported.

The `--debugger` variants launch the [Nix debugger](https://nix.dev/manual/nix/latest/command-ref/nix-build#opt-debugger), which drops into an interactive REPL on evaluation errors or `builtins.break` calls. Useful commands: `:bt` (backtrace), `:env` (show variables), `:continue`, `:step`.

### Configure formatters

You can configure formatters through LSP:

```jsonc
{
  "lsp": {
    "nil": {
      // or "nixd":
      "initialization_options": {
        "formatting": {
          "command": ["alejandra", "--quiet", "--"], // or ["nixfmt"]
        },
      },
    },
  },
}
```

Or through Zed itself:

```jsonc
{
  "languages": {
    "Nix": {
      "formatter": {
        "external": {
          "command": "alejandra", // or "nixfmt"
          "arguments": ["--quiet", "--"],
        },
      },
    },
  },
}
```
