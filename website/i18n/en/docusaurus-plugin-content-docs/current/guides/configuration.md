---
title: Configuration
description: Configure the popup, history suggestions, and candidate limit from the CLI.
---

## View preferences {#設定を表示する}

```bash
figfig config
figfig config get popup
figfig config path
```

`config` and `config list` print the current preferences as JSON. The `settings` alias supports the same operations.

## Available settings {#変更できる項目}

| Key | Value | Default | Purpose |
| --- | --- | --- | --- |
| `popup` | `true` / `false` | `true` | Show the completion popup |
| `history` | `true` / `false` | `true` | Suggest commands from zsh history |
| `max-candidates` | Integer from 3 to 12 | `8` | Maximum number of displayed candidates |

```bash
figfig config set popup true
figfig config set history false
figfig config set max-candidates 6
```

Saving preferences makes a running helper reload them and notify connected shells. You can also read and write preferences while the helper is stopped.

## Settings file {#設定ファイル}

The default location is shown below.

```text
~/Library/Application Support/figfig/settings.json
```

The JSON file uses these field names.

```json
{
  "popupEnabled": true,
  "historyEnabled": true,
  "maxCandidates": 8
}
```

The CLI validates values before saving. After editing the file directly, stop the running helper with `figfig stop` and open a new zsh session to reload it.

## Temporary pause {#一時停止}

```bash
figfig pause
figfig resume
```

Pause state is managed separately from saved preferences. Restarting the helper restores its normal state.

## Development environment variables {#開発検証用の環境変数}

| Variable | Purpose |
| --- | --- |
| `FIGFIG_CONFIG_DIR` | Directory containing `settings.json` |
| `FIGFIG_RUNTIME_DIR` | Directory containing the Unix socket |
| `FIGFIG_APP` | App bundle or executable that the CLI starts as its helper |
| `FIGFIG_TERMINAL_PID` | Override the terminal PID passed by the shell bridge |

When overriding locations, start the CLI and helper with the same inherited environment. The runtime directory must be owned by your user and have mode 0700.
