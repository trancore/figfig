---
title: Verification
description: Automated results, native checks, and remaining verification for the initial release.
---

## Popup UI results recorded on 2026-11-17 {#2026-11-17の補完ui確認結果}

- Updated the popup to a 320px-wide list with 20px rows, seven candidate-type colors, selection markers, and footer descriptions.
- All 3 UI tests passed with `pnpm test`, covering selection-driven descriptions and ARIA associations, an empty candidate list, and the `folder` fallback for directories without descriptions.
- `pnpm exec tsc --noEmit`, `pnpm exec vite build`, `cargo check -p figfig-desktop --locked --offline`, and `cargo fmt --all -- --check` passed.
- Safari previews covered 1, 4, and 12 candidates, selection changes, long-label and description truncation, and candidate-type colors.
- Rendering in the updated native Tauri window remains unverified. Browser previews do not verify positioning, focus, or insertion in a real terminal.

## Results recorded on 2026-11-16 {#2026-11-16の確認結果}

- TypeScript checking, 2 popup UI tests, 20 Rust tests, 5 CLI integration tests, 6 zsh integration tests, and 2 formula-generation tests were run on Apple Silicon and macOS.
- Workspace-wide Clippy with warnings treated as errors, production app generation, and ad-hoc signature verification were run.
- Refactoring separated CLI lifecycle, shell communication, setup, helper connections and state, and dynamic candidate generation. Regression tests cover NUL-delimited input, JSON frame limits, argument resolution, and late responses after disconnection.
- CLI integration was checked with disposable processes, covering automatic startup, preference storage and notification, pause and resume, setup backups, and duplicate prevention.
- `python3 scripts/smoke-cli.py` checked the real production helper, covering bridge-triggered startup, live preference notifications, pause and resume, diagnostics, completion-worker responses, and shutdown. The helper was stopped after the test.
- Basic `git ch` popup display in Terminal.app and iTerm2 was manually confirmed by the user before the CLI workflow change. Detailed native interactions require the checks below.
- Intel builds, installation, upgrades, and removal through a public tap, and detailed IME, multi-display, and full-screen behavior remain unverified. CI jobs and manual procedures are provided.

## Automated coverage {#自動検証の担当範囲}

| Level | Target | Dependencies | Command |
| --- | --- | --- | --- |
| Unit | Parsing, quoting, session state, late responses, frames, and CLI NUL-delimited input | Pure logic and in-memory streams | `cargo test -p figfig-core --lib`, `cargo test -p figfig-cli` |
| Component | Selection and footer-description associations, empty candidate lists, and directories without descriptions | React and jsdom | `pnpm test` |
| Integration | Paths, Git branches, and package scripts | Temporary directories and real Git | `cargo test -p figfig-core --test completions` |
| Integration | ZLE, Rust bridge, socket, Enter, Tab, arrows, history, and disconnects | Real zsh, PTY, Rust CLI, and a test backend | `pnpm run test:shell` |
| Integration | CLI setup, validation, preference notifications, pause, and automatic startup | Temporary HOME and ZDOTDIR, real CLI, and an isolated helper | `pnpm run test:shell` |
| Packaging | Versioned formula syntax and input validation | Python standard library and Ruby | Included in `pnpm run test:shell` |

After `pnpm run build:release`, stop any existing figfig helper and run `python3 scripts/smoke-cli.py` to check the real helper. It uses disposable configuration and requires macOS with a display. Real shell configuration and macOS permissions are preserved during this check.

These checks form the initial test foundation. Visual baselines depend on stable native layouts. Native permission, focus, and coordinate behavior require real-environment verification.

## Manual macOS checks {#macosでの操作確認}

Run `figfig setup` and `figfig permissions`, open a new shell, and perform the following checks in **both Terminal.app and iTerm2**. Record the observed behavior alongside automated results.

- Type `git ch` and verify that suggestions appear near the cursor while keyboard focus stays in the terminal.
- Select with arrows, insert with Tab or Enter, and execute with the next Enter. After Esc, suggestions should reappear when input is edited.
- Check contextual suggestions for `git status --s`, `git switch fe`, and `pnpm run te`.
- Create filenames containing Japanese characters, spaces, and quotes. Verify that insertion produces a single argument.
- Check Japanese IME composition, candidate selection, and confirmation with the expected input behavior.
- Check multiline input, wrapped lines, and replacement ranges while editing within a word.
- Switch between apps, tabs, and iTerm2 panes. Verify dismissal and new suggestions after editing at the destination.
- Move and resize windows, use full screen and mixed-scale displays, and check placement near the bottom edge of the screen.
- Remove permission, stop the helper, and disable the popup. Verify ordinary Enter, history, and Tab behavior in these states.
- Verify automatic helper startup from the shell and CLI-based control.
- Verify pause and resume, `config set history false`, preference saving, and loading after restart.
- Load figfig after Oh My Zsh and common prompt initialization. Disable overlapping completion integrations during these checks.

## Acceptance criteria {#完了条件}

Initial-release acceptance includes automated verification, production builds, ad-hoc signature checks, the manual checks in both terminals, and public Homebrew installation, upgrades, and removal after publication. Record unavailable native or distribution checks as unverified inputs to the release decision.
