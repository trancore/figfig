---
title: Development and builds
description: Set up the TypeScript and Rust environment, build figfig, and run its tests.
---

## Requirements {#必要な環境}

Use macOS with Node.js 24.15+ on the 24.x line, or 26+, pnpm 12.21.0, Rust 1.90 or later, and the Xcode Command Line Tools. Run commands from the root of your source checkout.

```bash
pnpm install --frozen-lockfile --ignore-scripts
```

Dependencies are recorded in `pnpm-lock.yaml` and `Cargo.lock`. Development uses Rust 1.100.0 through `rust-toolchain.toml`, which rustup selects inside the project.

Dependencies use the latest stable releases where compatible, and Docusaurus packages use matching versions. As of 2026-11-16, the Mermaid theme in Docusaurus 3.10.2 requires the ELK 0.1 series. Diagram rendering therefore uses Mermaid 11.17.2 and `@mermaid-js/layout-elk` 0.1.9.

## Start development {#開発時の起動}

```bash
pnpm run dev
```

To preview the React popup alone, run the following command and open `http://localhost:1420/?preview=1`.

```bash
pnpm run dev:ui
```

## Production build {#本番ビルド}

```bash
pnpm run build:release
```

| Artifact | Location |
| --- | --- |
| CLI | `target/release/figfig` |
| Popup helper | `target/release/bundle/macos/figfig.app` |

The build uses macOS ad-hoc signing. The CLI automatically starts the helper from the matching layout.

## Automated tests {#自動テスト}

```bash
pnpm run check
pnpm test
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build -p figfig-cli --locked
pnpm run test:shell
```

`test:shell` uses the built debug CLI. It tests integration with zsh, a PTY, a disposable helper, temporary home and configuration directories, and a Unix socket.

To test communication with the production helper, stop any existing figfig helper first.

```bash
python3 scripts/smoke-cli.py
```

This test requires macOS with a display. See the [verification record](../verification.md) for popup positioning and IME checks.

## Source layout {#ソースの構成}

| Location | Responsibility |
| --- | --- |
| `crates/core` | Input parsing, candidate generation, protocol types, settings, and session state |
| `crates/cli` | Commands, helper lifecycle, zsh setup, and shell communication |
| `src-tauri` | Connections, asynchronous completions, macOS positioning, and popup management |
| `src` | React popup |
| `shell/figfig.zsh` | ZLE input, key bindings, and history suggestions |
| `specs/index.ts` | Built-in command definitions |
| `website` | Docusaurus site configuration and presentation |
| `docs` | Japanese documentation source |
| `website/i18n/en/docusaurus-plugin-content-docs/current` | English documentation source |

See [Releasing](../releasing.md) for application distribution and [Documentation site](documentation.md) for website development and publishing.

## JetBrains plugin

Run `python3 scripts/build-jetbrains-plugin.py --ide '/Applications/IntelliJ IDEA.app'` to generate `target/editors/figfig-terminal.zip`. The installed IDE supplies the JDK and compile-time platform libraries. `test:shell` includes the real Java adapter IPC test when this SDK is available; set `FIGFIG_JETBRAINS_IDE` to another installed JetBrains app. Runners without an IDE SDK explicitly skip this test.
