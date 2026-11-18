---
title: Installation
description: Requirements, building figfig from source, zsh integration, and macOS permissions.
---

figfig is a CLI tool that adds a completion popup and inline history suggestions to zsh. Once you enable the shell integration, each new zsh session automatically starts the background popup helper.

## Requirements {#動作環境}

| Component | Supported environment |
| --- | --- |
| OS | macOS 13 or later |
| Shell | zsh 5.9 or later |
| Terminal | Terminal.app, iTerm2, or Zed's integrated terminal |
| Permission | Accessibility for Terminal.app/iTerm2; optional cursor tracker approval for Zed |

See [Compatibility](../reference/compatibility.md) for the supported scope and verification status.

## Build from source {#ソースから導入する}

The Homebrew tap `trancore/tap` (GitHub repository: `trancore/homebrew-tap`) is being prepared. For now, use the site's GitHub link to obtain the source, then run the build commands from the root of the figfig source directory.

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

## Use Zed's integrated terminal {#zedの内蔵ターミナルで使う}

After enabling the usual zsh integration, run:

```bash
figfig setup zed
```

This installs `~/Library/Input Methods/figfig Cursor.app`. Allow “figfig Cursor Tracking” when macOS asks to enable it, then restart Zed and open a new zsh terminal. If macOS has not recognized the registration, log out and back in, then retry the command.

The tracker is a palette input method. It requests cursor geometry in Zed without handling keystrokes, reading text, or changing your keyboard layout or Japanese input method. Suggestions use the same separate popup window.

Run `figfig setup zed` again after upgrading. Before removing the CLI, uninstall the optional tracker with `figfig setup zed --remove`.

## Verify the integration {#動作を確認する}

Open a new zsh session and type `git ch`. Select a suggestion and insert it with Tab or Enter. After inserting a suggestion, press Enter again to execute the command.

Check the connection and permissions with the following command.

```bash
figfig doctor
```

If suggestions do not appear, see [Troubleshooting](../guides/troubleshooting.md).

## Planned Homebrew installation {#homebrewでの導入予定}

After the first GitHub Release and formula are published, install with the following command. Until then, use the source build above.

```bash
brew install trancore/tap/figfig
figfig setup
figfig permissions
```

Homebrew also builds from source. Install the Xcode Command Line Tools first; Homebrew installs Node.js, pnpm, and Rust as build dependencies. The CLI is installed at `bin/figfig`, with the private helper at `libexec/figfig.app`.

To upgrade, run `figfig stop`, `brew update`, and `brew upgrade trancore/tap/figfig`, then open a new zsh session. macOS may ask you to grant permission again after an upgrade.

## Remove shell integration {#連携を解除する}

Run `figfig stop` to stop the helper, then remove the managed block from `# >>> figfig >>>` through `# <<< figfig <<<` in `.zshrc`. If you enabled integration manually, remove the initialization line you added.

To remove a source build, check and remove its CLI and helper files along with any PATH entry you added. Use `figfig config path` to locate saved preferences.

For a Homebrew installation, remove the shell integration first, then run `brew uninstall trancore/tap/figfig`. Saved preferences are retained.
