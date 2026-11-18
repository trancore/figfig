---
title: Releasing source builds
description: Prepare a tagged source release, verify Homebrew installation, and publish the formula automatically.
---

The source repository is `trancore/figfig`. Homebrew commands use `trancore/tap` for the `trancore/homebrew-tap` repository. The release workflow creates a source archive, checksum, and formula, then attaches them to a GitHub Draft Release. After publication, `Update Homebrew tap` verifies the public assets and updates the formula on the tap's `main` branch.

## Publication flow {#公開までの流れ}

The first publication is being prepared. See the [verification record](verification.md#2026-11-18のhomebrew公開準備の確認結果) for local validation results.

1. Publish the tap repository and configure its Actions secret in the source repository.
2. Tag the intended source commit and generate a Draft Release.
3. Install the draft's source through a disposable tap and verify it.
4. Publish the release as Latest and confirm that `Update Homebrew tap` succeeds.
5. Check both tap CI jobs and the official installation command.

Unless noted otherwise, run the following commands from the **figfig source root**. Keep `homebrew-tap` in the adjacent directory, and replace `v0.1.0` with the version being released.

## Publish the tap repository {#初回のtap公開}

The local `homebrew-tap` directory is an independent Git repository with publishing branch `main` and origin `git@github.com:trancore/homebrew-tap.git`. Add the first formula from actual release assets; do not commit placeholder URLs or checksums.

```sh
(
  set -e
  cd ../homebrew-tap || exit 1
  git add README.md LICENSE .gitignore .github
  git commit -m "chore: initialize Homebrew tap"
  gh repo create trancore/homebrew-tap --public
  git push -u origin HEAD:main
)
```

After the parenthesized commands finish, your shell remains at the source root. The initial push sends the working branch HEAD to the public `main` branch. Skip `gh repo create` if the GitHub repository already exists. Before a formula is added, tap CI reports that the release is pending and skips installation checks.

### Set up authentication for tap updates

Use an account that can push to `main` in `trancore/homebrew-tap`. Create a fine-grained personal access token scoped to `trancore/homebrew-tap` with the `Contents` repository permission set to `Read and write`. [GitHub's token setup guide](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)

In `trancore/figfig`, open **Settings → Secrets and variables → Actions** and save the token as a repository secret named `HOMEBREW_TAP_TOKEN`. Use the same secret name when rotating the token. [GitHub's Actions secret setup guide](https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets)

## Create a Draft Release from a tag {#タグからdraft-releaseを作成する}

1. Align versions in `package.json`, workspace `Cargo.toml`, and `src-tauri/tauri.conf.json`, and update `Cargo.lock`. The first version is `0.1.0`.
2. Complete source CI, production builds, and [native verification](verification.md). Commit and push the intended source changes.
3. Create and push a matching tag:

```sh
git tag -a v0.1.0 -m "figfig v0.1.0"
git push origin v0.1.0
```

`Prepare release` checks out the tagged commit and verifies versions and generated specifications. Only committed files enter the archive. Assets are saved in the `source-and-homebrew-formula` Actions artifact and a Draft Release for the same tag.

| Asset | Contents |
| --- | --- |
| `figfig-0.1.0-source.tar.gz` | Source from the tagged commit; no compiled binaries |
| `SHA256SUMS` | SHA-256 of the source archive |
| `figfig.rb` | Formula with the public release URL and checksum, published to `Formula/figfig.rb` in the tap |
| `release-notes.md` | Source commit and installation steps; stored in the Actions artifact and used as the draft body |

To run the workflow manually for an existing tag:

```sh
gh workflow run release.yml --repo trancore/figfig -f tag=v0.1.0
```

The workflow does not overwrite an existing release. If one already exists, draft creation fails; inspect the existing draft and assets. Never move a published tag.

To prepare assets locally, use a working copy checked out at the intended tag. Python 3.11+ is required.

```sh
python3 scripts/prepare-release.py --repository trancore/figfig --tag v0.1.0 --output release
ruby -c release/Formula/figfig.rb
(cd release && shasum -a 256 -c SHA256SUMS)
```

## Verify Homebrew before publishing {#公開前にhomebrewで検証する}

Download the draft assets into a temporary directory:

```sh
release_dir="$(mktemp -d)"
gh release download v0.1.0 --repo trancore/figfig --dir "$release_dir" \
  --pattern figfig-0.1.0-source.tar.gz --pattern SHA256SUMS --pattern figfig.rb
(cd "$release_dir" && shasum -a 256 -c SHA256SUMS)
```

Keep the public formula unchanged. From the source root, generate a separate formula that references the local archive and install it from a test tap. Use a test Mac without another Homebrew installation of figfig.

```sh
source_sha="$(shasum -a 256 "$release_dir/figfig-0.1.0-source.tar.gz" | awk '{print $1}')"
python3 scripts/generate-formula.py \
  --repository trancore/figfig --version 0.1.0 --sha256 "$source_sha" \
  --source-url "file://$release_dir/figfig-0.1.0-source.tar.gz" \
  --output "$release_dir/tap/Formula/figfig.rb"
git -C "$release_dir/tap" init
git -C "$release_dir/tap" add Formula/figfig.rb
git -C "$release_dir/tap" commit -m "test: local source release"
brew tap --custom-remote trancore/figfig-validation "$release_dir/tap"
brew style trancore/figfig-validation/figfig
brew audit --strict trancore/figfig-validation/figfig
brew install --build-from-source trancore/figfig-validation/figfig
brew test trancore/figfig-validation/figfig
```

`brew test` checks the CLI version, completion output, zsh initialization, helper placement, and ad-hoc signature. It does not grant Accessibility permission or verify native interaction. On a test Mac, check `figfig setup`, `figfig permissions`, automatic startup in a new shell, configuration, pause, and input in Terminal.app and iTerm2 separately.

After testing, stop the helper, remove any shell integration you added, and remove the test installation and tap:

```sh
figfig stop
brew uninstall trancore/figfig-validation/figfig
brew untap trancore/figfig-validation
```

## Publish the release and update the tap {#release公開とtap更新}

1. Review the draft assets and verification results, then publish the release as Latest on GitHub.
2. `Update Homebrew tap` downloads the source archive and attachments from their public URLs. It verifies the archive against `SHA256SUMS`, checks the formula's URL, SHA-256 and Ruby syntax, then pushes the formula to the tap's `main` branch.
3. Check that tap CI passes syntax, style, metadata, source installation, formula tests, and removal on Apple Silicon and Intel.
4. On a test Mac, verify the official installation command, upgrades to a later version, and removal. Record unavailable checks in [Verification](verification.md).

```sh
brew install trancore/tap/figfig
brew test trancore/tap/figfig
figfig setup
figfig permissions
```

To recover from a failed update or sync an already published release, run `Update Homebrew tap` manually from the source repository's default branch. Both automatic and manual runs sync the stable release GitHub currently marks as Latest. An identical formula needs no new commit.

```sh
gh workflow run update-homebrew-tap.yml --repo trancore/figfig
```

After the first official installation is verified, remove the pending-release notices from the tap README, source README, both installation pages, and translated homepage text. Update both compatibility pages and verification records with the actual results.

Repeat this process with a new tag for each update. Existing users run `figfig stop`, `brew update`, and `brew upgrade trancore/tap/figfig`, then open a new zsh session.

## Layout and removal {#配置と削除}

The formula installs the CLI at `bin/figfig` and the private helper at `libexec/figfig.app`. The CLI locates the matching helper from its resolved executable path and starts it in the background. No copy to `/Applications` or manual app launch is required.

Before uninstalling, run `figfig stop` and remove the managed block from `# >>> figfig >>>` through `# <<< figfig <<<` in `.zshrc` (or the manually added initialization line), then run `brew uninstall trancore/tap/figfig`. Saved preferences are retained.

Distribution builds from source with an ad-hoc signature. Bottles, Developer ID signing, notarized binaries, and Homebrew core registration are outside this release scope. Keep Gatekeeper and quarantine protections enabled during verification.
