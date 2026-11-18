---
title: Compatibility
description: Supported operating systems, terminals, shell syntax, and verification status.
---

## Initial release targets {#初期版の対象}

| Component | Supported scope |
| --- | --- |
| OS | macOS 13 or later |
| Shell | zsh 5.9 or later |
| Terminal | Terminal.app, iTerm2, VSCode, Cursor, Zed, and major JetBrains IDEs |
| Keymaps | emacs and vi insert mode |
| Candidate sources | Local command definitions, PATH, files, Git branches, and package scripts |

SSH sessions, tmux, other shells, and other terminals are outside the initial release scope.

## Integrated terminal requirements

| Host | Position/focus provider | Verification |
| --- | --- | --- |
| VSCode stable / Insiders, Cursor | Accessibility; optional InputMethodKit fallback | Registry, native geometry, and PTY ancestry tests; real popup acceptance pending |
| Zed stable / Preview / Nightly | InputMethodKit (`figfig setup terminal`) | Registry, native geometry, IPC, and ancestry tests; real popup acceptance pending |
| IntelliJ IDEA, PyCharm, WebStorm, GoLand, CLion, Rider, RubyMine, DataGrip, RustRover, Android Studio | figfig Terminal plugin | Plugin build against installed IntelliJ SDK and real Java IPC tested; per-product acceptance pending |

JetBrains Classic and Reworked are targets; deprecated Experimental 2024 is excluded. Product recognition is not evidence that all versions and terminal engines have passed real UI checks. See the [installation guide](../getting-started/installation.md#zedの内蔵ターミナルで使う).

## Input parsing {#入力の解析}

The parser supports literal input with spaces, quotes, and Unicode, along with input after pipes and command separators. File candidates are shell-quoted when inserted, including names containing spaces or shell metacharacters.

Command substitution, variable expansion, and redirection are outside the supported parsing scope. Your original zsh bindings remain available for input that produces no candidates.

## Command definitions {#コマンド定義}

Built-in definitions cover major subcommands and options for git, brew, npm, pnpm, and docker. Dynamic Git candidates are local branches. Dynamic package-manager candidates are package scripts in the working directory.

See [Adding command definitions](../development/specifications.md) to define additional commands.

## Permission and focus {#権限フォーカス}

Terminal.app/iTerm2 use macOS Accessibility. VSCode/Cursor first use Accessibility geometry after verifying the focused xterm input class, with the shared InputMethodKit tracker as a fallback. Zed uses that tracker; JetBrains uses its plugin. The popup is shown when a supported terminal is in the foreground and its PID matches the shell's terminal PID.

## Verification status {#検証状況}

Integrated-terminal coordinate conversion, stale-response rejection, native tracker IPC, editor-host ancestry, setup aliases, and the JetBrains plugin socket are covered by automated tests. Real popup placement, IME use, and pane switching require separate acceptance checks.

Automated tests, production builds, signing, and communication with the real helper have been checked on Apple Silicon and macOS. Basic popup display in Terminal.app and iTerm2 was manually confirmed before the CLI-only workflow was introduced.

Homebrew distribution is being prepared. Source builds, installation, formula audits, tests, and removal through a local validation tap were verified on Apple Silicon and macOS 27.0.1. Installation from a public release through the official tap remains a post-publication check. See the [release guide](../releasing.md) for the procedure.

Intel builds, detailed IME behavior, mixed-scale displays, full-screen operation, and installation, upgrades, and removal through a public tap remain unverified. See the [verification record](../verification.md) for results and procedures.
