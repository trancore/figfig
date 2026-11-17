# ソースビルド版のリリース

本体は `trancore/figfig`、tapリポジトリは `trancore/homebrew-tap`、Homebrewで指定するtap名は `trancore/tap` です。タグからソースtar.gz、SHA-256、Formulaを生成し、GitHubのDraft Releaseへ添付します。公開内容を確認してReleaseを公開した後、Formulaをtapへ手動反映します。tapへの自動pushや自動PR作成は行いません。

## 公開までの流れ

初回公開は準備中です。ローカルでの検証結果は[検証記録](verification.md#2026-11-18のhomebrew公開準備の確認結果)を参照してください。

1. tapリポジトリを公開する。
2. 本体の配布対象コミットへタグを付け、Draft Releaseを生成する。
3. Draftのソースを検証専用tapからインストールして確認する。
4. Releaseを公開し、公開URLからソースのSHA-256を確認する。
5. Releaseに添付されたFormulaをtapへ反映し、両CPUのCIと正式な導入コマンドを確認する。

以下のコマンドは、特記がない限り **figfig本体のルートディレクトリ** から実行します。`homebrew-tap` は隣接ディレクトリに配置し、`v0.1.0` は公開するバージョンに合わせて読み替えてください。

## 初回のtap公開

`homebrew-tap` ディレクトリは独立したGitリポジトリです。公開ブランチは `main`、originは `git@github.com:trancore/homebrew-tap.git` です。初回FormulaはRelease成果物から追加するため、公開前に仮のURL・チェックサムを配置しません。

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

括弧内の処理が終わると、実行場所は本体のルートのままです。初回は作業ブランチのHEADを公開先の `main` へ送ります。既にGitHubリポジトリを作成済みの場合は、`gh repo create` を省略します。Formula追加前のtap CIは、未公開であることを通知してインストール検証を省略します。

## タグからDraft Releaseを作成する

1. `package.json`、ワークスペースの `Cargo.toml`、`src-tauri/tauri.conf.json` のバージョンを揃え、`Cargo.lock` も更新します。初回は `0.1.0` です。
2. 本体CI、ソースからのアプリ生成、[ネイティブ操作の確認](verification.md)を完了し、配布対象の変更をコミット・pushします。
3. バージョンタグを作成してpushします。

```sh
git tag -a v0.1.0 -m "figfig v0.1.0"
git push origin v0.1.0
```

`Prepare release` がタグのコミットをチェックアウトし、バージョンと生成済み仕様を検証します。ソースはそのコミットだけから作成し、ローカルの未コミット変更は含めません。成果物はActionsの `source-and-homebrew-formula` と、同じタグのDraft Releaseに保存します。

| 成果物 | 内容 |
| --- | --- |
| `figfig-0.1.0-source.tar.gz` | タグのコミットから生成したソース。ビルド済みバイナリは含まない |
| `SHA256SUMS` | ソースアーカイブのSHA-256 |
| `figfig.rb` | 公開ReleaseのURLとSHA-256を指定したFormula。tapでは `Formula/figfig.rb` に配置 |
| `release-notes.md` | 元のコミットと導入手順。Actions成果物に保存し、Draftの本文に使用 |

既存タグを指定して手動実行することもできます。

```sh
gh workflow run release.yml --repo trancore/figfig -f tag=v0.1.0
```

既存Releaseの上書きは行いません。同じタグのReleaseが存在する場合、Draft作成は失敗します。既存Draftと成果物を確認し、公開済みタグを付け替えないでください。

生成処理だけをローカルで実行する場合は、対象タグをチェックアウトした作業用コピーで次を実行します。Python 3.11以降が必要です。

```sh
python3 scripts/prepare-release.py --repository trancore/figfig --tag v0.1.0 --output release
ruby -c release/Formula/figfig.rb
(cd release && shasum -a 256 -c SHA256SUMS)
```

## 公開前にHomebrewで検証する

Draft Releaseから成果物を一時ディレクトリへダウンロードします。

```sh
release_dir="$(mktemp -d)"
gh release download v0.1.0 --repo trancore/figfig --dir "$release_dir" \
  --pattern figfig-0.1.0-source.tar.gz --pattern SHA256SUMS --pattern figfig.rb
(cd "$release_dir" && shasum -a 256 -c SHA256SUMS)
```

公開用Formulaは変更せず、ローカルファイルを参照する検証専用Formulaを別tapに生成します。本体のルートから実行してください。既に別のfigfigをHomebrewで導入している場合は、その環境ではなく検証用Macを使用します。

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

`brew test` はCLIのバージョン、補完、zsh初期化、ヘルパー配置、ad-hoc署名を確認します。アクセシビリティ許可や画面操作は自動テストでは確認できません。検証用Macで `figfig setup`、`figfig permissions`、新しいシェルでの自動起動、設定・一時停止、Terminal.app／iTerm2での入力を別途確認してください。

検証終了後はヘルパーを停止し、追加したzsh連携を削除してから検証用パッケージとtapを削除します。

```sh
figfig stop
brew uninstall trancore/figfig-validation/figfig
brew untap trancore/figfig-validation
```

## Release公開とtap更新

1. Draft Releaseのソースと確認結果をレビューし、GitHubで公開します。
2. 公開URLからソースを取得し直し、チェックサムを検証します。Draftの間はこの公開URLは使えません。

```sh
curl --fail --location \
  https://github.com/trancore/figfig/releases/download/v0.1.0/figfig-0.1.0-source.tar.gz \
  --output "$release_dir/figfig-0.1.0-source.tar.gz"
(cd "$release_dir" && shasum -a 256 -c SHA256SUMS)
mkdir -p ../homebrew-tap/Formula
cp "$release_dir/figfig.rb" ../homebrew-tap/Formula/figfig.rb
```

3. Formulaを対象にコミットし、pushします。

```sh
git -C ../homebrew-tap switch main
git -C ../homebrew-tap diff -- Formula/figfig.rb
git -C ../homebrew-tap add Formula/figfig.rb
git -C ../homebrew-tap commit -m "chore: publish figfig 0.1.0 formula"
git -C ../homebrew-tap push origin main
```

4. tapのCIでApple Silicon・Intelの構文・スタイル・メタデータ・ソースビルド・テスト・削除が成功したことを確認します。
5. 検証用Macで正式な導入コマンドと、次のバージョンでの `brew upgrade`、`brew uninstall` を確認します。未確認の項目は[検証記録](verification.md)へ残します。

```sh
brew install trancore/tap/figfig
brew test trancore/tap/figfig
figfig setup
figfig permissions
```

初回の正式導入が確認できたら、tapのREADME、本体README、日英のインストールページ、サイトトップの日英文言の「公開準備中」を削除し、日英の対応環境・検証記録を実際の結果で更新します。

更新時も新しいタグで同じ手順を繰り返します。既存ユーザーは `figfig stop`、`brew update`、`brew upgrade trancore/tap/figfig` の順に実行し、新しいzshセッションを開きます。

## 配置と削除

CLIは `bin/figfig`、内部ヘルパーは `libexec/figfig.app` へ配置します。CLIは自身の実パスから同じバージョンのヘルパーを見つけ、バックグラウンド起動します。`/Applications` へのコピーや手動起動は不要です。

アンインストール前に `figfig stop` を実行し、`.zshrc` の `# >>> figfig >>>` から `# <<< figfig <<<` までの管理用ブロック（手動設定の場合は初期化行）を削除してから、`brew uninstall trancore/tap/figfig` を実行します。ユーザー設定は削除しません。

配布はソースビルド・ad-hoc署名です。bottle、Developer ID署名・公証済みバイナリ、Homebrew公式coreへの登録は含みません。Gatekeeperやquarantineを無効化せずに検証してください。
