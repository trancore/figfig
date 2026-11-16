---
title: Privacy and permissions
description: How figfig handles input, history, settings, and macOS Accessibility permission.
---

## Input and completion data {#入力と候補}

The shell sends the current command input, working directory, cursor position, and terminal PID to a local helper through a user-private Unix socket. The helper uses these values to generate candidates and locate the popup.

figfig operates locally during completion. Its implementation has no external completion requests, AI generation, login, or telemetry.

## History {#履歴}

History suggestions run inside zsh. History storage follows your zsh configuration, with no separate figfig history file.

## Files and Git {#ファイルとgit}

File completion reads the path being typed. Git branch completion uses a read-only Git command with a 150ms timeout. Package-script completion reads script names and descriptions from `package.json`.

## Accessibility permission {#アクセシビリティ権限}

figfig uses this permission to inspect the foreground terminal, focused input element, and cursor position. Grant it through macOS System Settings after running `figfig permissions`.

macOS Accessibility permission allows access to other applications' UI information. You can manage figfig's permission in the operating system's settings.

## Storage and logs {#保存先とログ}

Settings are stored in `~/Library/Application Support/figfig/settings.json` by default. Use `figfig config path` to inspect the active location.

The Unix socket lives in a user-owned runtime directory with mode 0700. The socket itself is created with mode 0600. Logs record errors such as connection failures. Input buffers, working directories, and history are excluded from logs.

See [Architecture](../architecture.md) for the implementation details.
