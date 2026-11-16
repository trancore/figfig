---
title: CLI reference
description: figfig commands, arguments, configuration keys, and exit codes.
---

## Setup and integration {#導入連携}

| Command | Action |
| --- | --- |
| `figfig setup` | Add integration to `.zshrc` and back up an existing file |
| `figfig init zsh` | Print the zsh integration script to standard output |
| `figfig permissions` | Start the helper and check or request macOS permission |

See the [installation guide](../getting-started/installation.md) for the full procedure.

## Configuration {#設定}

```bash
figfig config
figfig config list
figfig config path
figfig config get KEY
figfig config set KEY VALUE
```

| Key | Accepted values |
| --- | --- |
| `popup` | `true` / `false` |
| `history` | `true` / `false` |
| `max-candidates` | 3–12 |

`figfig settings` accepts the same arguments as `figfig config`. See [Configuration](../guides/configuration.md) for defaults and storage locations.

## Runtime control {#実行状態}

| Command | Action |
| --- | --- |
| `figfig start` | Start the background popup helper or use its existing connection |
| `figfig stop` | Shut down the helper |
| `figfig pause` | Start the helper and pause popup and history suggestions |
| `figfig resume` | Start the helper and resume popup and history suggestions |
| `figfig doctor` | Diagnose the connection, permission, and cursor positioning |

Normal zsh integration starts the helper automatically. Opening a new zsh session can restart it after `stop`.

## Inspect completions {#補完結果を調べる}

```bash
figfig complete TEXT [DIRECTORY]
```

Print up to 12 candidates as JSON, using the end of the input string as the cursor position. The directory defaults to your current working directory.

```bash
figfig complete 'git status --s'
figfig complete 'pnpm run ' /path/to/project
```

The response fields `replaceStart` and `replaceEnd` describe a replacement range in Unicode scalar offsets. See [Architecture](../architecture.md) for the protocol types.

## Help and version {#ヘルプとバージョン}

```bash
figfig --help
figfig -h
figfig --version
figfig -V
```

Running figfig without arguments also displays help.

## Internal shell communication {#内部のシェル通信}

`figfig bridge SESSION_ID` is the persistent bridge started by zsh integration. It reads NUL-delimited input and communicates with the helper over a Unix socket. Use `setup` or `init zsh` for normal shell integration.

## Exit codes {#終了コード}

Successful operations return `0`. Invalid arguments, settings errors, and connection failures return `1`. `doctor` also returns `1` when Accessibility permission is missing.
