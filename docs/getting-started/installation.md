---
title: インストール
description: figfigの動作環境、Homebrewでの導入、ソースからのビルド、zsh連携とmacOSの権限設定。
---

figfigはzshに補完ポップアップと履歴に基づくインライン予測を追加するCLIツールです。導入後は、新しいzshセッションから描画ヘルパーが自動起動します。

## 動作環境

| 項目 | 対応環境 |
| --- | --- |
| OS | macOS 13以降 |
| シェル | zsh 5.9以降 |
| ターミナル | Terminal.app、iTerm2、VSCode・Cursor・Zed・JetBrains系の内蔵ターミナル |
| 権限 | Terminal.app／iTerm2／VSCode／Cursorはアクセシビリティ。Zedはアクセシビリティ、カーソル位置に合わせた表示には共通ヘルパー、JetBrains系は専用プラグイン |

対応範囲と確認状況は[対応環境](../reference/compatibility.md)に記載しています。

## Homebrewで導入する {#homebrewでの導入予定}

Homebrew でインストールし、zsh連携とmacOSの権限を設定します。

```bash
brew install trancore/tap/figfig
figfig setup
figfig permissions
```

Homebrew版もソースからビルドします。Xcode Command Line Toolsが必要で、Node.js・pnpm・RustはHomebrewがビルド用依存として導入します。CLIは `bin/figfig`、ヘルパーは `libexec/figfig.app` に配置されます。

更新時は `figfig stop`、`brew update`、`brew upgrade trancore/tap/figfig` の順に実行し、新しいzshセッションを開いてください。更新後にmacOSの権限の再設定が必要になる場合があります。

## ソースから導入する

GitHubからソースを取得し、figfigソースのルートディレクトリでビルドします。

Node.js 24.15以降の24系、または26以降、pnpm 12.21.0、Rust 1.90以降、Xcode Command Line Toolsを用意してください。

```bash
pnpm install --frozen-lockfile --ignore-scripts
pnpm run build:release
./target/release/figfig setup
./target/release/figfig permissions
```

CLIは`target/release/figfig`、描画ヘルパーは`target/release/bundle/macos/figfig.app`に生成されます。CLIが同じ配布物のヘルパーを見つけるため、この配置を保ってください。以降の例で使う`figfig`は、ソースビルドでは`./target/release/figfig`へ読み替えられます。

普段のシェルからコマンド名で使う場合は、ビルドディレクトリをPATHへ追加します。ソースのルートディレクトリで次のコマンドを実行し、表示された行をシェル設定へ追加してください。

```bash
printf 'export PATH=%q:$PATH\n' "$PWD/target/release"
```

## zsh連携を有効にする

`figfig setup`はzsh設定へ管理用ブロックを追加します。

- `ZDOTDIR`が設定されている場合は、そのディレクトリの`.zshrc`を使います。
- 既存の`.zshrc`は、初回の変更時に`.zshrc.figfig-backup`へコピーします。
- 管理用ブロックまたは同じ初期化行がある場合は、再実行時の追記を省略します。

figfigはプロンプトやキーバインドの設定後に読み込んでください。

設定を手動で管理する場合は、`.zshrc`の末尾へ次の一行を追加できます。

```bash
eval "$(figfig init zsh)"
```

## macOSの権限を設定する

```bash
figfig permissions
```

開いたシステム設定でfigfigを有効にします。macOSのバージョンによって、項目は「アクセシビリティ」または「デバイスの制御とデータへのアクセス」と表示されます。権限を許可すると、最前面のターミナルの入力位置を取得して候補を配置できます。

## VSCode・Cursor・Zedの内蔵ターミナルで使う {#zedの内蔵ターミナルで使う}

VSCode・Cursor・Zedには共通のカーソル取得ヘルパーを用意しています。

```bash
figfig setup terminal
```

`~/Library/Input Methods/figfig Cursor.app`へパレット型入力メソッドを導入します。キーイベントや入力テキストを取得せず、キーボード配列や日本語入力を切り替えません。

初回登録直後は、macOSが入力メソッドを登録しても有効化しないことがあります。`figfig doctor` に `registered but disabled` と表示された場合は、**macOSからログアウト・ログイン**してから同じ `figfig setup terminal` を再実行してください。Zedだけの再起動ではこの状態は解消しません。その後エディタを再起動し、新しいローカルzshターミナルで確認します。

パレット型入力メソッドは通常の「キーボード → 入力ソース」には表示されず、有効化のための標準UIもありません。表示されない許可ダイアログを探す必要はありません。[Appleの説明](https://developer.apple.com/library/archive/qa/qa1810/_index.html)も参照してください。

VSCode・Cursorでは `figfig permissions` でアクセシビリティ権限も許可してください。ターミナル入力欄を識別し、取得できる場合はAccessibilityから直接位置を取得します。取得できない場合は共通ヘルパーを使います。Zedで画面上のカーソル座標を取得できないときは、下部ターミナルが返すカーソルの行・列とアクセシビリティから位置を推定し、入力の下に同じポップアップを表示します。`figfig permissions` で許可してください。上下キーで選び、TabまたはEnterで挿入できます。

更新後は同じコマンドを再実行します。削除は `figfig setup terminal --remove` です。既存の `figfig setup zed` と `figfig setup zed --remove` も同じ共有ヘルパーを操作する互換コマンドとして使えます。

## JetBrains系の内蔵ターミナルで使う

通常の `figfig setup` に加え、figfig Terminalプラグインを導入します。プラグインはターミナルのフォーカスとカーソル座標・行高、IME変換状態だけを扱い、入力文字列やキーを補完ヘルパーへ送りません。

ソースのルートで、導入済みのJetBrains IDEを指定してビルドします。IDEに付属するJDKとライブラリを使い、SDKの追加ダウンロードは行いません。

```bash
python3 scripts/build-jetbrains-plugin.py --ide '/Applications/IntelliJ IDEA.app'
```

IDEのSettings → Plugins → 歯車 → Install Plugin from Diskから `target/editors/figfig-terminal.zip` を選び、IDEを再起動してプロジェクトとローカルzshターミナルを開きます。ClassicとReworkedを検証対象とし、廃止済みのExperimental 2024は対象外です。組み込みの補完候補が重なる場合はIDE側のターミナル補完を無効にしてください。

IntelliJ IDEA、PyCharm、WebStorm、GoLand、CLion、Rider、RubyMine、DataGrip、RustRover、Android Studioを識別します。プラグインはビルド243以降のターミナルプラグインを持つIDE向けです。製品ごとの実機確認状況は[対応環境](../reference/compatibility.md)を参照してください。

JetBrainsの座標取得には入力メソッドの登録は不要です。`FIGFIG_RUNTIME_DIR`を変更する開発環境では、IDEとfigfigの両方を同じ値で起動します。削除はIDEのPluginsからfigfig Terminalをアンインストールします。

## 動作を確認する

新しいzshセッションを開き、`git ch`と入力します。候補を選び、TabまたはEnterで挿入してください。候補を挿入した後、次のEnterでコマンドを実行します。

接続・権限を診断するには次を実行します。

```bash
figfig doctor
```

候補が表示されない場合は[トラブルシューティング](../guides/troubleshooting.md)を参照してください。

## 連携を解除する

`figfig stop`でヘルパーを停止し、`.zshrc`の`# >>> figfig >>>`から`# <<< figfig <<<`までの管理用ブロックを削除します。手動で連携した場合は、追加した初期化行を削除してください。

ソースビルド版を削除する場合は、使用中のCLI・ヘルパーの配置と、追加したPATH設定を確認して取り除きます。保存済みの設定の場所は`figfig config path`で確認できます。

Homebrew版は連携を解除した後、`brew uninstall trancore/tap/figfig` で削除します。保存済みのユーザー設定は保持します。
