# ソースビルド版のリリース

本体は `trancore/figfig`、tapリポジトリは `trancore/homebrew-tap`、Homebrewで指定するtap名は `trancore/tap` です。タグからソースtar.gz、SHA-256、Formulaを生成し、GitHubのDraft Releaseへ添付します。Releaseを公開すると、`Update Homebrew tap` が公開成果物を検証し、tapの `main` にFormulaを反映します。

## 公開までの流れ

初回リリースは v0.0.1 です。確認結果は[検証記録](verification.md)を参照してください。

1. tapリポジトリを公開し、本体にtap更新用のActionsシークレットを設定する。
2. 本体の配布対象コミットへタグを付け、Draft Releaseを生成する。
3. Draftのソースを検証専用tapからインストールして確認する。
4. ReleaseをLatestとして公開し、`Update Homebrew tap` の成功を確認する。
5. tapの両CPUのCIと正式な導入コマンドを確認する。

以下のコマンドは、特記がない限り **figfig本体のルートディレクトリ** から実行します。`homebrew-tap` は隣接ディレクトリに配置し、`v0.0.1` は公開するバージョンに合わせて読み替えてください。

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

### tap更新用の認証

`trancore/homebrew-tap` の `main` にpushできるアカウントで、対象リポジトリを `trancore/homebrew-tap` に限定し、リポジトリ権限の `Contents` を `Read and write` にしたfine-grained personal access tokenを作成します。[GitHubのトークン作成手順](https://docs.github.com/ja/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)

`trancore/figfig` の **Settings → Secrets and variables → Actions** で、リポジトリシークレット `HOMEBREW_TAP_TOKEN` として登録します。トークンの更新時も同じ名前で登録します。[GitHubのシークレット設定手順](https://docs.github.com/ja/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets)

## タグからDraft Releaseを作成する

1. `package.json`、ワークスペースの `Cargo.toml`、`src-tauri/tauri.conf.json` のバージョンを揃え、`Cargo.lock` も更新します。初回は `0.0.1` です。
2. 本体CI、ソースからのアプリ生成、[ネイティブ操作の確認](verification.md)を完了し、配布対象の変更をコミット・pushします。
3. バージョンタグを作成してpushします。

```sh
git tag -a v0.0.1 -m "figfig v0.0.1"
git push origin v0.0.1
```

`Prepare release` がタグのコミットをチェックアウトし、バージョンと生成済み仕様を検証します。ソースはそのコミットだけから作成し、ローカルの未コミット変更は含めません。成果物はActionsの `source-and-homebrew-formula` と、同じタグのDraft Releaseに保存します。

| 成果物 | 内容 |
| --- | --- |
| `figfig-0.0.1-source.tar.gz` | タグのコミットから生成したソース。ビルド済みバイナリは含まない |
| `SHA256SUMS` | ソースアーカイブのSHA-256 |
| `figfig.rb` | 公開ReleaseのURLとSHA-256を指定したFormula。公開後にtapの `Formula/figfig.rb` へ自動反映 |
| `release-notes.md` | 元のコミットと導入手順。Actions成果物に保存し、Draftの本文に使用 |

既存タグを指定して手動実行することもできます。

```sh
gh workflow run release.yml --repo trancore/figfig -f tag=v0.0.1
```

既存Releaseの上書きは行いません。同じタグのReleaseが存在する場合、Draft作成は失敗します。既存Draftと成果物を確認し、公開済みタグを付け替えないでください。

生成処理だけをローカルで実行する場合は、対象タグをチェックアウトした作業用コピーで次を実行します。Python 3.11以降が必要です。

```sh
python3 scripts/prepare-release.py --repository trancore/figfig --tag v0.0.1 --output release
ruby -c release/Formula/figfig.rb
(cd release && shasum -a 256 -c SHA256SUMS)
```

## 公開前にHomebrewで検証する

Draft Releaseから成果物を一時ディレクトリへダウンロードします。

```sh
release_dir="$(mktemp -d)"
gh release download v0.0.1 --repo trancore/figfig --dir "$release_dir" \
  --pattern figfig-0.0.1-source.tar.gz --pattern SHA256SUMS --pattern figfig.rb
(cd "$release_dir" && shasum -a 256 -c SHA256SUMS)
```

公開用Formulaは変更せず、ローカルファイルを参照する検証専用Formulaを別tapに生成します。本体のルートから実行してください。既に別のfigfigをHomebrewで導入している場合は、その環境ではなく検証用Macを使用します。

```sh
source_sha="$(shasum -a 256 "$release_dir/figfig-0.0.1-source.tar.gz" | awk '{print $1}')"
python3 scripts/generate-formula.py \
  --repository trancore/figfig --version 0.0.1 --sha256 "$source_sha" \
  --source-url "file://$release_dir/figfig-0.0.1-source.tar.gz" \
  --output "$release_dir/tap/Formula/figfig.rb"
git -C "$release_dir/tap" init
git -C "$release_dir/tap" add Formula/figfig.rb
git -C "$release_dir/tap" commit -m "test: local source release"
brew tap --custom-remote trancore/figfig-validation "$release_dir/tap"
brew trust --formula trancore/figfig-validation/figfig
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
brew untrust --formula trancore/figfig-validation/figfig
brew untap trancore/figfig-validation
```

## Release公開とtap更新

1. Draft Releaseのソースと確認結果をレビューし、GitHubでLatestとして公開します。
2. 本体の `Update Homebrew tap` が公開URLからソースと添付ファイルを取得します。ソースの実測SHA-256を `SHA256SUMS` と照合し、Formulaに記載されたURLとSHA-256、Rubyの構文を確認してから、tapの `main` にpushします。
3. tapのCIでApple Silicon・Intelの構文・スタイル・メタデータ・ソースビルド・テスト・削除が成功したことを確認します。
4. 検証用Macで正式な導入コマンドと、次のバージョンでの `brew upgrade`、`brew uninstall` を確認します。未確認の項目は[検証記録](verification.md)へ残します。

```sh
brew install trancore/tap/figfig
brew test trancore/tap/figfig
figfig setup
figfig permissions
```

失敗からの復旧や、すでに公開したReleaseの反映には `Update Homebrew tap` を本体のデフォルトブランチから手動実行できます。自動・手動実行ともにその時点でGitHubがLatestとして返す安定版が対象です。Formulaが同一なら新しいコミットは作りません。

```sh
gh workflow run update-homebrew-tap.yml --repo trancore/figfig
```

公開後は、日英の対応環境と検証記録を実際の結果に合わせて更新します。

更新時も新しいタグで同じ手順を繰り返します。既存ユーザーは `figfig stop`、`brew update`、`brew upgrade trancore/tap/figfig` の順に実行し、新しいzshセッションを開きます。

## 配置と削除

CLIは `bin/figfig`、内部ヘルパーは `libexec/figfig.app` へ配置します。CLIは自身の実パスから同じバージョンのヘルパーを見つけ、バックグラウンド起動します。`/Applications` へのコピーや手動起動は不要です。

アンインストール前に `figfig stop` を実行し、`.zshrc` の `# >>> figfig >>>` から `# <<< figfig <<<` までの管理用ブロック（手動設定の場合は初期化行）を削除してから、`brew uninstall trancore/tap/figfig` を実行します。ユーザー設定は削除しません。

配布はソースビルド・ad-hoc署名です。bottle、Developer ID署名・公証済みバイナリ、Homebrew公式coreへの登録は含みません。Gatekeeperやquarantineを無効化せずに検証してください。
