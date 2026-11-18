---
title: Verification
description: Automated results, native checks, and remaining verification for the initial release.
---

## Integrated terminals, 2026-11-18

Rust (27), integration (27), and UI (3) tests passed on Apple Silicon/macOS 27.0.1, along with Clippy, TypeScript checks, and both documentation builds. Integration covers real host ancestry through PTY helpers, shared setup aliases in disposable homes, native geometry IPC, legacy coordinate frames, and stale-response rejection. The JetBrains plugin ZIP was compiled against IntelliJ IDEA 2025.3.2/JDK 21. Its real Java socket fixture checks zero-height normalization, pane identity, composition state, search exclusion, malformed/wrong-PID/insecure-socket rejection, and cleanup. The fixture supplies geometry rather than running an IDE.

The final production build and ad-hoc signature verification passed. The packaged helper passed seven CLI smoke checks: automatic startup, preference updates, pause/resume, doctor, invalid cursor-registration path rejection, completion responses, and shutdown. Normal helper startup was restored afterward; the real `.zshrc` was unchanged.

VSCode's terminal input was observed, but automated input did not produce a verifiable screen change. The updated InputMethodKit tracker also needs macOS activation, which was refused during setup. Log out/in, retry `figfig setup terminal`, and restart the editor. Real popup placement/insertion in VSCode/Cursor/Zed, JetBrains Classic/Reworked, product-specific compatibility, IME, pane switching, zoom, and mixed-scale displays remain unverified. Registry and fixture coverage do not constitute real-IDE acceptance.

## Homebrew preparation results recorded on 2026-11-18 {#2026-11-18のhomebrew公開準備の確認結果}

- Built and installed a source archive of commit `d4e29633a41f5b22564b0c12b8e682665315116e` through a disposable Homebrew tap on Apple Silicon and macOS 27.0.1. The local validation formula was kept out of the public tap directory.
- The final formula passed `brew style`, `brew audit --strict`, and `brew test`, covering the CLI version, completion, zsh initialization, helper placement at `libexec/figfig.app`, and ad-hoc signature verification.
- Uninstallation and test-tap removal succeeded. The real `.zshrc` and macOS permissions were unchanged.
- All 19 tests passed: 5 CLI, 6 zsh, 3 formula-generation, and 5 release-preparation tests. Disposable Git repositories cover tag/commit/version mismatch rejection, consistency between the archive, SHA-256, and formula, repeatable archives, and exclusion of uncommitted files.
- Existing PTY and Unix-socket tests require execution outside the sandbox. They ran with disposable homes and test processes.
- actionlint checks for the release and tap workflows, website type checking, and both locale builds passed.
- This formula's Intel build, GitHub Draft Release creation, installation through the public tap, upgrades between versions, and native terminal interaction for this revision remain unverified. Check them in CI and release acceptance after publication.

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
