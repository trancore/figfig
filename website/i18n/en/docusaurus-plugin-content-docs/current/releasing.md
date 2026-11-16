---
title: Releasing source builds
description: Prepare a versioned source archive and Homebrew formula, then verify installation and upgrades.
---

Application distribution uses a public repository and a `homebrew-tap`. After preparing those destinations, follow the steps below to publish the source build.

1. Align the versions in `package.json`, the workspace `Cargo.toml`, and `src-tauri/tauri.conf.json`, and update lockfiles.
2. Complete CI, source builds, and [native verification](verification.md).
3. Create a version tag such as `v0.1.0` and run the `Prepare release` workflow.
4. Download the source tarball, `SHA256SUMS`, and `Formula/figfig.rb` from the workflow's `source-and-homebrew-formula` artifact.
5. Create a GitHub Release for the same tag and attach the source tarball and `SHA256SUMS` using their original filenames.
6. Place the formula at `Formula/figfig.rb` in your tap repository. Verify that its source URL can be downloaded.
7. On a test Mac, run `brew install --build-from-source OWNER/tap/figfig` and `brew test OWNER/tap/figfig`. Verify setup, permissions, automatic startup in a new shell, CLI configuration and pause, input in both terminals, `brew upgrade`, and `brew uninstall`.

You can also generate the formula manually using the repository, version, and source archive's SHA-256.

```bash
python3 scripts/generate-formula.py \
  --repository OWNER/figfig \
  --version 0.1.0 \
  --sha256 ACTUAL_SOURCE_ARCHIVE_SHA256
ruby -c Formula/figfig.rb
```

For local distribution checks, use `--source-url file:///absolute/path/to/source.tar.gz` and place the formula in a test tap. Before publishing, replace `OWNER` and the checksum with actual release values.

The formula installs the CLI at `bin/figfig` and the private popup helper at `libexec/figfig.app`. The CLI finds its matching helper from its own resolved path and starts it in the background. Before uninstalling, run `figfig stop` and remove the initialization line from `.zshrc`. Saved preferences are retained.

Current distribution uses local builds and ad-hoc signing. Developer ID signing, notarized binaries, and Homebrew core registration are outside this release scope. Keep Gatekeeper and quarantine protections enabled during verification.
