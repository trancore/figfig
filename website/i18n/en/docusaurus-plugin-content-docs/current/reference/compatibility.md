---
title: Compatibility
description: Supported operating systems, terminals, shell syntax, and verification status.
---

## Initial release targets {#初期版の対象}

| Component | Supported scope |
| --- | --- |
| OS | macOS 13 or later |
| Shell | zsh 5.9 or later |
| Terminal | Terminal.app, iTerm2, and Zed's integrated terminal |
| Keymaps | emacs and vi insert mode |
| Candidate sources | Local command definitions, PATH, files, Git branches, and package scripts |

SSH sessions, tmux, other shells, and other terminals are outside the initial release scope.

## Input parsing {#入力の解析}

The parser supports literal input with spaces, quotes, and Unicode, along with input after pipes and command separators. File candidates are shell-quoted when inserted, including names containing spaces or shell metacharacters.

Command substitution, variable expansion, and redirection are outside the supported parsing scope. Your original zsh bindings remain available for input that produces no candidates.

## Command definitions {#コマンド定義}

Built-in definitions cover major subcommands and options for git, brew, npm, pnpm, and docker. Dynamic Git candidates are local branches. Dynamic package-manager candidates are package scripts in the working directory.

See [Adding command definitions](../development/specifications.md) to define additional commands.

## Permission and focus {#権限フォーカス}

Cursor position comes from macOS Accessibility. The popup is shown when a supported terminal is in the foreground and its PID matches the shell's terminal PID.

## Verification status {#検証状況}

Zed requires the optional InputMethodKit cursor tracker installed with `figfig setup zed`. Stable, Preview, and Nightly app processes are recognized. Geometry conversion, stale-response rejection, and native tracker IPC are covered by automated tests. Actual popup placement, IME use, and pane switching with the registered tracker require manual verification.

Automated tests, production builds, signing, and communication with the real helper have been checked on Apple Silicon and macOS. Basic popup display in Terminal.app and iTerm2 was manually confirmed before the CLI-only workflow was introduced.

Homebrew distribution is being prepared. Source builds, installation, formula audits, tests, and removal through a local validation tap were verified on Apple Silicon and macOS 27.0.1. Installation from a public release through the official tap remains a post-publication check. See the [release guide](../releasing.md) for the procedure.

Intel builds, detailed IME behavior, mixed-scale displays, full-screen operation, and installation, upgrades, and removal through a public tap remain unverified. See the [verification record](../verification.md) for results and procedures.
