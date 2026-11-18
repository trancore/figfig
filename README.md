# figfig

※ このアプリはバイブコーディングによって作成されています。

zshにFig風の補完ポップアップと、履歴に基づくインライン予測を追加するCLIツールです。導入・設定・一時停止はすべて`figfig`コマンドで操作します。

**対象：macOS 13以降、zsh 5.9以降。** Terminal.app／iTerm2に加え、VSCode・Cursor・Zed・主要JetBrains IDEの内蔵ターミナルを識別します。エディタ連携の実機確認状況は[対応環境](docs/reference/compatibility.md)を参照してください。

VSCode・Cursorは `figfig permissions` でアクセシビリティを許可し、位置取得の補助には `figfig setup terminal` を使います。Zedでは `figfig permissions` を使うと、下部ターミナルで入力カーソルの下にポップアップを表示できます。`figfig setup terminal` で位置取得用ヘルパーを有効にすると、カーソルのそばに表示できます。従来の `figfig setup zed` も利用できます。共有ヘルパーの削除は `figfig setup terminal --remove` です。

JetBrains系ではfigfig Terminalプラグインを使います。`python3 scripts/build-jetbrains-plugin.py` で `target/editors/figfig-terminal.zip` を生成し、IDEのPluginsから導入します。詳しい[内蔵ターミナルの導入手順](docs/getting-started/installation.md#jetbrains系の内蔵ターミナルで使う)を参照してください。

## 導入

Homebrew tap `trancore/tap`（GitHubリポジトリ: `trancore/homebrew-tap`）は公開準備中です。初回ReleaseとFormulaの公開後は、次の流れで導入できます。

現在すぐに試す場合は、[ソースからビルド](#ソースからビルド)を利用してください。公開作業は[リリース手順](docs/releasing.md)にまとめています。

```sh
brew install trancore/tap/figfig
figfig setup
figfig permissions
```

Homebrew版もソースからビルドするため、Xcode Command Line Toolsが必要です。Node.js・pnpm・RustはHomebrewがビルド用依存として導入します。

`figfig setup`は`.zshrc`の末尾にzsh連携を追加します。既存ファイルは`.zshrc.figfig-backup`へ保存し、再実行しても初期化行を重複させません。`ZDOTDIR`を設定している場合は、そのディレクトリの`.zshrc`を使います。

`figfig permissions`からmacOSの設定を開き、figfigを有効にしてください。項目名は「アクセシビリティ」または「デバイスの制御とデータへのアクセス」です。権限の付与自体はmacOSの操作が必要です。

新しいzshセッションを開くと有効になります。**アプリを手動で開いたり、設定ウィンドウやメニューバーを操作する必要はありません。** ポップアップ描画用のヘルパーはシェルからバックグラウンドで自動起動します。

`.zshrc`を手動で管理したい場合は、`setup`の代わりに次の一行を末尾へ追加します。

```sh
eval "$(figfig init zsh)"
```

## 設定と操作

```sh
figfig config                         # 現在の設定をJSONで表示
figfig config get history              # 履歴予測の設定を確認
figfig config set history false        # 履歴予測を無効化
figfig config set popup true           # ポップアップを有効化
figfig config set max-candidates 6     # 候補の上限を変更（3〜12）
figfig config path                    # 設定ファイルの場所
figfig pause                          # 一時停止
figfig resume                         # 再開
figfig doctor                         # 接続・権限・位置取得の診断
```

設定変更は保存され、実行中のシェルにも通知します。一時停止は現在のヘルパーが終了するまで有効です。設定は`~/Library/Application Support/figfig/settings.json`に保存します。旧コマンド`figfig settings`もCLIの設定表示として使用できます。

`figfig start`と`figfig stop`で表示ヘルパーを手動管理することもできます。通常は自動起動に任せます。停止後に新しいシェルセッションを開くと再起動するため、継続して無効にする場合は設定または`.zshrc`の初期化行を変更してください。

| キー        | 候補表示中     | 通常時                 |
| ----------- | -------------- | ---------------------- |
| ↑ / ↓       | 候補選択       | 元の履歴・カーソル操作 |
| Tab         | 選択候補を挿入 | figfigの候補一覧が有効なら何もしない |
| Enter       | 選択候補を挿入 | 元のzsh操作            |
| Esc         | 候補を閉じる   | 元のzsh操作            |
| 行末の→     | 履歴予測を確定 | 元のzsh操作            |

**Enterでの候補挿入ではコマンドを実行せず、次のEnterで実行します。** 挿入・閉じる操作の後は、入力を編集するまで候補を再表示しません。

候補一覧を無効にしたとき、一時停止中、ヘルパーとの接続が切れたときは、Tabは元のzsh操作に戻ります。

候補はコンパクトな1行リストで表示し、種類ごとに候補名とアイコンを7色で色分けします。選択中の候補は背景色と左端のマーカーで示し、説明は下部のフッターに表示します。見方とキー操作は[基本操作](docs/guides/usage.md)を参照してください。

コマンド名・パス・主要なgit／brew／npm／pnpm／dockerのサブコマンドとオプションを補完します。gitのローカルブランチと、現在のディレクトリの`package.json`内のscriptsも候補になります。

```sh
figfig complete 'git ch'        # 補完エンジンの結果をJSONで確認
figfig complete 'pnpm run ' .   # package.jsonのスクリプトを確認
```

## ソースからビルド

Node.js 24.15以降の24系、または26以降、pnpm 12.21.0、Rust 1.90以降、Xcode Command Line Toolsが必要です。pnpmのバージョンは`packageManager`、依存バージョンは`pnpm-lock.yaml`と`Cargo.lock`で固定しています。

```sh
pnpm install --frozen-lockfile --ignore-scripts
pnpm run build:release
./target/release/figfig setup
./target/release/figfig permissions
```

CLIは`target/release/figfig`、描画ヘルパーは`target/release/bundle/macos/figfig.app`に生成されます。後者はCLIが内部で起動するものです。Homebrew配布時もユーザー向けの導入口は`bin/figfig`で、ヘルパーは`libexec`へ配置します。

開発時は`pnpm run dev`でポップアップ用ヘルパーを起動します。表示部分だけをブラウザで確認する場合は`pnpm run dev:ui`の後に`http://localhost:1420/?preview=1`を開きます。

## 制約・プライバシー

- AI、ログイン、補完時の外部通信、テレメトリー、履歴の独自保存はありません。
- 入力バッファと作業ディレクトリはユーザー専用のUnixソケットでローカルのヘルパーへ渡します。履歴はzsh内で処理し、ヘルパーへ送りません。
- gitブランチの取得は読み取り専用です。入力文字列やpackage scriptsを補完時に実行しません。
- コマンド置換やリダイレクトなど、未対応の構文では候補を表示しません。任意のFig仕様・zsh補完関数の互換実装ではありません。
- ポップアップを表示できない状態は `figfig doctor` で診断できます。Zedでは、下部ターミナルが返すカーソルの行・列と推定した表示領域を使ってポップアップを配置できます。行・列を取得できない場合は、前面のZedウィンドウ下部に表示します。
- SSH先・tmux内・他のシェルとターミナルは初期版の対象外です。
- `zsh-autosuggestions`、`zsh-autocomplete`、Fig／Amazon Qなど、同じ入力やキーを扱う機能は無効にし、Oh My Zshなどの初期化後に読み込んでください。
- ヘルパーはローカルのad-hoc署名を使用し、Developer IDによる署名・公証はしていません。更新後に権限の再設定が必要になる場合があります。

公開手順は[リリース手順](docs/releasing.md)、確認済み・未確認の項目は[検証記録](docs/verification.md)、内部構成は[アーキテクチャ](docs/architecture.md)を参照してください。

## テストとコマンド定義

```sh
pnpm run check
pnpm test
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build -p figfig-cli --locked
pnpm run test:shell
```

CLI・zshの統合テストでは、一時的なHOME／ZDOTDIR、テスト用バックグラウンドプロセス、PTY、Unixソケットを使います。実際の`.zshrc`やmacOSの権限は変更しません。

コマンド定義はTypeScriptの`specs/index.ts`で編集し、`pnpm run specs`でRustが読む`assets/specs.json`を生成します。生成JSONもソースへ含めるため、インストール後の利用にNode.jsやpnpmは不要です。`FIGFIG_CONFIG_DIR`、`FIGFIG_RUNTIME_DIR`、`FIGFIG_APP`はテスト・開発時の場所の切り替えに使用できます。

## 公開ドキュメント

Docusaurusサイトは英語・日本語に対応しています。日本語原稿は`docs/`、英語原稿とUI翻訳は`website/i18n/en/`、サイト設定は`website/`にあります。

```sh
pnpm --dir website install --frozen-lockfile --ignore-scripts
pnpm run docs:dev
```

`http://localhost:3000/`で確認できます。英語版は`pnpm run docs:dev --locale en`で起動し、`http://localhost:3000/en/`を開きます。ビルド・GitHub Pagesへの公開は[ドキュメントサイトの運用](docs/development/documentation.md)を参照してください。

## English

figfig is a CLI tool that adds Fig-style popup completions and history suggestions to zsh. Run `figfig setup`, grant macOS permission using `figfig permissions`, then open a new shell. The private popup renderer starts automatically in the background, without a settings window, menu bar application, or manual app launch.

The compact popup displays one candidate per row and uses seven colors for candidate labels and icons. A tinted background and a left-edge marker identify the selection, while its description appears in the footer. See [Basic usage](website/i18n/en/docusaurus-plugin-content-docs/current/guides/usage.md) for the color key and keyboard controls.

Use `figfig config`, `figfig config set history false`, `figfig pause`, `figfig resume`, and `figfig doctor` from your terminal. Setup preserves the existing `.zshrc`, creates a backup, and avoids duplicate entries. Settings are persistent and propagated to connected shells; pause is temporary.

Build with `pnpm install --frozen-lockfile --ignore-scripts` and `pnpm run build:release`. The Homebrew formula installs the CLI in `bin` and its native popup helper privately in `libexec`. The public tap `trancore/tap` (GitHub repository: `trancore/homebrew-tap`) is being prepared. After publication, install with `brew install trancore/tap/figfig`. Until then, use the source build above. Maintainers can follow the [release guide](website/i18n/en/docusaurus-plugin-content-docs/current/releasing.md). Recognized macOS hosts include Terminal.app, iTerm2, VSCode, Cursor, Zed, and major JetBrains IDEs. VSCode/Cursor use Accessibility with an optional `figfig setup terminal` tracker fallback; Zed uses the tracker to anchor the popup to the cursor and Accessibility and the terminal’s reported cursor cell to position it beneath the input in the bottom terminal when the tracker is unavailable. JetBrains uses the figfig Terminal plugin built with `python3 scripts/build-jetbrains-plugin.py`. See the installation guide and compatibility matrix for requirements and verification status. SSH, tmux, other shells, and AI are outside this release.

MIT License.
