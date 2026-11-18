---
title: Troubleshooting
description: Diagnose missing suggestions, conflicting key bindings, and helper startup failures.
---

## Start with diagnostics {#最初に確認する}

```bash
figfig --version
figfig start
figfig doctor
```

`doctor` reports runtime and settings locations, protocol version, pause state, Accessibility permission, the foreground terminal, and position-tracking diagnostics. Missing permission produces exit code 1.

## Suggestions do not appear {#候補が表示されない}

1. Check [Compatibility](../reference/compatibility.md).
2. Open a new zsh session after running `figfig setup`.
3. Check that `figfig config get popup` returns `true`.
4. Run `figfig resume` if suggestions are paused.
5. Run `figfig permissions` and enable figfig in System Settings.
6. Return to Terminal.app or iTerm2 and type `git ch`.

If `figfig complete 'git ch'` returns entries in `candidates`, completion data is being generated. Check the position and permission information in `doctor` when diagnosing popup display.

In Zed, figfig estimates the bottom terminal’s grid origin from Accessibility window bounds and uses the reported cursor cell to put the popup below the input when caret tracking is unavailable. It places the popup above the input if it would extend below the screen. When the cursor report or grid estimate is unavailable, it places the popup near the bottom of the focused window. Grant access with `figfig permissions`. A `Popup positioning: Zed window` diagnosis confirms window placement is available. Use `figfig setup terminal` to enable the tracker for caret placement.

## The helper cannot be found {#ヘルパーが見つからない}

For source builds, check that both of the following paths exist.

```text
target/release/figfig
target/release/bundle/macos/figfig.app
```

Run `pnpm run build:release` from the source root to generate both the CLI and helper. Homebrew packaging uses `bin/figfig` and `libexec/figfig.app`.

## History suggestions do not appear {#履歴予測が表示されない}

Check `figfig config get history`. History suggestions require nonempty input, a cursor at the end of the line, and an editing mode other than vi command mode. Check that the current zsh history contains a command starting with your input.

## Key bindings or prompts conflict {#キー操作やプロンプトが競合する}

Load figfig after initializing Oh My Zsh and your prompt. Check overlapping integrations such as `zsh-autosuggestions`, `zsh-autocomplete`, Fig, or Amazon Q that handle the same input or keys.

Use `figfig pause` to help isolate the problem. The [installation guide](../getting-started/installation.md#連携を解除する) explains how to remove integration.

## The settings file produces an error {#設定ファイルのエラーが出る}

Use `figfig config path` to locate the file, then match its JSON syntax and keys to the [configuration example](configuration.md#設定ファイル). CLI configuration commands require an existing settings file to be readable when one is present.

## Suggestions disappear when returning to the terminal {#ターミナルへ戻ると候補が消えている}

figfig closes the popup when focus moves to another app, tab, or pane, or when cursor-position lookup fails. Editing the input in the target terminal sends a new request.

## Permission is no longer recognized after an update {#更新後に権限が認識されない}

Current builds use ad-hoc signing. If an updated bundle requires permission, run `figfig permissions` and check the figfig entry in System Settings.

When reporting a reproducible issue, include your macOS, terminal, zsh, and figfig versions, along with minimal reproduction steps. Review diagnostic output for personal paths such as your home directory before sharing it.

## Integrated terminal adapters

VSCode/Cursor require Accessibility permission and a focused terminal input. If direct geometry is unavailable, run `figfig setup terminal`, restart the editor, and log out/in first if the tracker is registered but disabled. For JetBrains, install figfig Terminal and open a project. A missing `jb-<PID>.sock` indicates that the plugin is not running or the IDE and helper use different `FIGFIG_RUNTIME_DIR` values. Check `figfig doctor` from the affected terminal. Disable overlapping editor terminal completion if both systems display suggestions.
