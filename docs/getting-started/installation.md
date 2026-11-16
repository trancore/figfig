---
title: インストール
description: figfigの動作環境、ソースからのビルド、zsh連携とmacOSの権限設定。
---

figfigはzshに補完ポップアップと履歴に基づくインライン予測を追加するCLIツールです。導入後は、新しいzshセッションから描画ヘルパーが自動起動します。

## 動作環境

| 項目 | 対応環境 |
| --- | --- |
| OS | macOS 13以降 |
| シェル | zsh 5.9以降 |
| ターミナル | Terminal.app、iTerm2 |
| 権限 | 入力位置を取得するためのアクセシビリティ権限 |

対応範囲と確認状況は[対応環境](../reference/compatibility.md)に記載しています。

## ソースから導入する

Homebrewの公開tapは準備中です。サイトのGitHubリンクからソースを取得し、figfigソースのルートディレクトリでビルドします。

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

Oh My Zshなどの初期化後に連携を読み込んでください。同じ入力・キーを扱うFig、Amazon Q、`zsh-autosuggestions`、`zsh-autocomplete`などの機能を使用している場合は、重複する連携を無効にします。

設定を手動で管理する場合は、`.zshrc`の末尾へ次の一行を追加できます。

```bash
eval "$(figfig init zsh)"
```

## macOSの権限を設定する

```bash
figfig permissions
```

開いたシステム設定でfigfigを有効にします。macOSのバージョンによって、項目は「アクセシビリティ」または「デバイスの制御とデータへのアクセス」と表示されます。権限を許可すると、最前面のターミナルの入力位置を取得して候補を配置できます。

## 動作を確認する

新しいzshセッションを開き、`git ch`と入力します。候補を選び、TabまたはEnterで挿入してください。候補を挿入した後、次のEnterでコマンドを実行します。

接続・権限を診断するには次を実行します。

```bash
figfig doctor
```

候補が表示されない場合は[トラブルシューティング](../guides/troubleshooting.md)を参照してください。

## Homebrewでの導入予定

公開tapを用意した後、配布先のtap名を指定して導入できる構成です。公開先が決まり次第、このページへ実際のインストールコマンドを掲載します。

## 連携を解除する

`figfig stop`でヘルパーを停止し、`.zshrc`の`# >>> figfig >>>`から`# <<< figfig <<<`までの管理用ブロックを削除します。手動で連携した場合は、追加した初期化行を削除してください。

ソースビルド版を削除する場合は、使用中のCLI・ヘルパーの配置と、追加したPATH設定を確認して取り除きます。保存済みの設定の場所は`figfig config path`で確認できます。
