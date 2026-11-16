# ソースビルド版のリリース

このリポジトリには、公開先を自動で決める処理や、外部リポジトリへの自動投稿・pushはありません。公開リポジトリと`homebrew-tap`を用意してから、以下の手順で配布します。

1. `package.json`、ワークスペースの`Cargo.toml`、`src-tauri/tauri.conf.json`のバージョンを揃え、ロックファイルを更新します。
2. CI、ソースからのアプリ生成、[ネイティブ操作の確認](verification.md)を完了します。
3. バージョンタグ（例：`v0.1.0`）を作成し、`Prepare release`ワークフローを実行します。
4. ワークフローの`source-and-homebrew-formula`成果物から、ソースtar.gz、`SHA256SUMS`、`Formula/figfig.rb`を取得します。
5. 同じタグのGitHub Releaseを作成し、ファイル名を変えずにソースtar.gzと`SHA256SUMS`を添付します。
6. Formulaを独自tapリポジトリの`Formula/figfig.rb`に配置します。FormulaのURLが実際に取得できることを確認します。
7. 検証用Macで`brew install --build-from-source OWNER/tap/figfig`、`brew test OWNER/tap/figfig`を実行し、`figfig setup`、`figfig permissions`、新しいシェルでの自動起動、CLIによる設定・一時停止、両ターミナルでの入力、`brew upgrade`、`brew uninstall`を確認します。

Formulaは公開先のリポジトリ名、バージョン、ソースのSHA-256を引数に手動でも生成できます。

```sh
python3 scripts/generate-formula.py \
  --repository OWNER/figfig \
  --version 0.1.0 \
  --sha256 ACTUAL_SOURCE_ARCHIVE_SHA256
ruby -c Formula/figfig.rb
```

ローカル配布検証では`--source-url file:///absolute/path/to/source.tar.gz`を指定できます。生成したFormulaをテスト専用tapに配置してください。`OWNER`やSHA-256が未確定のFormulaを公開しないでください。

CLIはFormulaの`bin/figfig`、内部のポップアップ描画ヘルパーは`libexec/figfig.app`へ配置します。CLIは自身の実パスから同じバージョンのヘルパーを見つけ、自動でバックグラウンド起動します。`/Applications`へのコピーや手動でのアプリ起動は不要です。アンインストール前に`figfig stop`を実行し、`.zshrc`の初期化行を削除してください。ユーザー設定は削除しません。

現段階の配布はローカルビルド・ad-hoc署名です。Developer ID署名・公証済みバイナリやHomebrew公式coreへの登録は含みません。起動許可のためにGatekeeperやquarantineを無効化する操作は行いません。
