---
title: Installation
description: Requirements, Homebrew installation, building figfig from source, zsh integration, and macOS permissions.
---

figfig is a CLI tool that adds a completion popup and inline history suggestions to zsh. Once you enable the shell integration, each new zsh session automatically starts the background popup helper.

## Requirements {#動作環境}

| Component | Supported environment |
| --- | --- |
| OS | macOS 13 or later |
| Shell | zsh 5.9 or later |
| Terminal | Terminal.app, iTerm2, and VSCode/Cursor/Zed/JetBrains integrated terminals |
| Permission | Accessibility for Terminal.app/iTerm2/VSCode/Cursor; Accessibility for a window-anchored popup in Zed, optional tracker for caret placement; plugin for JetBrains |

See [Compatibility](../reference/compatibility.md) for the supported scope and verification status.

## Install with Homebrew {#homebrewでの導入予定}

Install figfig with Homebrew, then set up zsh integration and macOS permissions.

```bash
brew install trancore/tap/figfig
figfig setup
figfig permissions
```

Homebrew also builds from source. Install the Xcode Command Line Tools first; Homebrew installs Node.js, pnpm, and Rust as build dependencies. The CLI is installed at `bin/figfig`, with the private helper at `libexec/figfig.app`.

To upgrade, run `figfig stop`, `brew update`, and `brew upgrade trancore/tap/figfig`, then open a new zsh session. macOS may ask you to grant permission again after an upgrade.

## Build from source {#ソースから導入する}

Get the source from GitHub, then run the build commands from the root of the figfig source directory.

Install Node.js 24.15+ on the 24.x line, or 26+, pnpm 12.21.0, Rust 1.90 or later, and the Xcode Command Line Tools.

```bash
pnpm install --frozen-lockfile --ignore-scripts
pnpm run build:release
./target/release/figfig setup
./target/release/figfig permissions
```

The CLI is generated at `target/release/figfig` and the popup helper at `target/release/bundle/macos/figfig.app`. Keep this layout so the CLI can locate its matching helper. In the examples below, you can replace `figfig` with `./target/release/figfig` when using a source build.

To use the command by name from your regular shell, add the build directory to PATH. From the source root, run the following command and add the printed line to your shell configuration.

```bash
printf 'export PATH=%q:$PATH\n' "$PWD/target/release"
```

## Enable zsh integration {#zsh連携を有効にする}

`figfig setup` adds a managed block to your zsh configuration.

- If `ZDOTDIR` is set, it uses `.zshrc` in that directory.
- On the first modification, an existing `.zshrc` is copied to `.zshrc.figfig-backup`.
- If the managed block or the same initialization line already exists, running setup again skips the append.

Load figfig after configuring your prompt and key bindings.

If you manage your configuration manually, add this line at the end of `.zshrc`.

```bash
eval "$(figfig init zsh)"
```

## Grant macOS permission {#macosの権限を設定する}

```bash
figfig permissions
```

Enable figfig in the System Settings panel that opens. Depending on your macOS version, the permission appears as Accessibility or Device Control and Data Access. This allows figfig to read the cursor position in the foreground terminal and place suggestions beside it.

## Use VSCode, Cursor, and Zed integrated terminals {#zedの内蔵ターミナルで使う}

VSCode, Cursor, and Zed share an optional cursor tracker:

```bash
figfig setup terminal
```

This installs a palette input method at `~/Library/Input Methods/figfig Cursor.app`. It does not handle keystrokes, read input text, or switch your keyboard layout or Japanese input method.

Immediately after a first installation, macOS may register the tracker without enabling it. If `figfig doctor` reports `registered but disabled`, **log out of macOS and back in**, then repeat `figfig setup terminal`. Restarting Zed alone does not resolve this state. Then restart the editor and open a new local zsh terminal.

Palette input methods do not appear in Keyboard → Input Sources, and macOS provides no standard UI to enable them. Do not wait for an activation dialog that has not appeared. See [Apple's explanation](https://developer.apple.com/library/archive/qa/qa1810/_index.html).

VSCode and Cursor also require Accessibility permission through `figfig permissions` to distinguish the terminal input from the editor. They use Accessibility geometry directly when available, with InputMethodKit as a fallback. In Zed, figfig uses InputMethodKit for caret placement when available. With Accessibility permission from `figfig permissions`, it estimates the grid origin and uses the reported cursor cell to show the same popup below the input in the bottom terminal when the tracker is unavailable. Use Up and Down to select and Tab or Enter to insert.

Repeat setup after updating. Remove the shared tracker with `figfig setup terminal --remove`. The existing `figfig setup zed` and `figfig setup zed --remove` remain compatibility aliases for the same bundle.

## Use JetBrains integrated terminals

After the usual `figfig setup`, install the figfig Terminal plugin. It shares only terminal focus, cursor geometry, and line height with the local helper, without forwarding text or keystrokes. Composition state is checked locally to suppress suggestions during IME conversion.

Build from the source root against an installed IDE. The script uses its bundled JDK and libraries without downloading an SDK:

```bash
python3 scripts/build-jetbrains-plugin.py --ide '/Applications/IntelliJ IDEA.app'
```

In Settings → Plugins → gear → Install Plugin from Disk, select `target/editors/figfig-terminal.zip`. Restart the IDE and open a project and a local zsh terminal. Classic and Reworked are verification targets; deprecated Experimental 2024 is excluded. Disable the IDE's built-in terminal completion if it overlaps figfig.

Recognized products are IntelliJ IDEA, PyCharm, WebStorm, GoLand, CLion, Rider, RubyMine, DataGrip, RustRover, and Android Studio. The plugin targets build 243 or later with the terminal plugin installed. See [Compatibility](../reference/compatibility.md) for product-specific verification status.

JetBrains cursor positioning does not require the input method. If using `FIGFIG_RUNTIME_DIR` for development, launch both the IDE and figfig with the same value. Uninstall figfig Terminal from Plugins to remove the adapter.


## Verify the integration {#動作を確認する}

Open a new zsh session and type `git ch`. Select a suggestion and insert it with Tab or Enter. After inserting a suggestion, press Enter again to execute the command.

Check the connection and permissions with the following command.

```bash
figfig doctor
```

If suggestions do not appear, see [Troubleshooting](../guides/troubleshooting.md).

## Remove shell integration {#連携を解除する}

Run `figfig stop` to stop the helper, then remove the managed block from `# >>> figfig >>>` through `# <<< figfig <<<` in `.zshrc`. If you enabled integration manually, remove the initialization line you added.

To remove a source build, check and remove its CLI and helper files along with any PATH entry you added. Use `figfig config path` to locate saved preferences.

For a Homebrew installation, remove the shell integration first, then run `brew uninstall trancore/tap/figfig`. Saved preferences are retained.
