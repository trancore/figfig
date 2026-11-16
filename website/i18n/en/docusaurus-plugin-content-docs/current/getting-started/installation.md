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
| Terminal | Terminal.app or iTerm2 |
| Permission | Accessibility access to read the terminal cursor position |

See [Compatibility](../reference/compatibility.md) for the supported scope and verification status.

## Build from source {#ソースから導入する}

The public Homebrew tap is being prepared. Use the site's GitHub link to obtain the source, then run the build commands from the root of the figfig source directory.

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

Load the integration after Oh My Zsh or other shell initialization. Disable overlapping integrations from Fig, Amazon Q, `zsh-autosuggestions`, or `zsh-autocomplete` if they handle the same input or keys.

If you manage your configuration manually, add this line at the end of `.zshrc`.

```bash
eval "$(figfig init zsh)"
```

## Grant macOS permission {#macosの権限を設定する}

```bash
figfig permissions
```

Enable figfig in the System Settings panel that opens. Depending on your macOS version, the permission appears as Accessibility or Device Control and Data Access. This allows figfig to read the cursor position in the foreground terminal and place suggestions beside it.

## Verify the integration {#動作を確認する}

Open a new zsh session and type `git ch`. Select a suggestion and insert it with Tab or Enter. After inserting a suggestion, press Enter again to execute the command.

Check the connection and permissions with the following command.

```bash
figfig doctor
```

If suggestions do not appear, see [Troubleshooting](../guides/troubleshooting.md).

## Planned Homebrew installation {#homebrewでの導入予定}

The packaging supports installation through a named Homebrew tap. This page will include the actual installation command once the public tap is available.

## Remove shell integration {#連携を解除する}

Run `figfig stop` to stop the helper, then remove the managed block from `# >>> figfig >>>` through `# <<< figfig <<<` in `.zshrc`. If you enabled integration manually, remove the initialization line you added.

To remove a source build, check and remove its CLI and helper files along with any PATH entry you added. Use `figfig config path` to locate saved preferences.
