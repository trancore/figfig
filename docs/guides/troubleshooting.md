---
title: トラブルシューティング
description: 補完が表示されない、キーが競合する、ヘルパーが起動しない場合の確認手順。
---

## 最初に確認する

```bash
figfig --version
figfig start
figfig doctor
```

`doctor`には、ランタイムと設定の場所、通信プロトコル、一時停止、アクセシビリティ権限、最前面のターミナル、位置取得の診断が表示されます。権限が不足すると終了コード1になります。

## 候補が表示されない

1. [対応環境](../reference/compatibility.md)を確認します。
2. `figfig setup`の後、新しいzshセッションを開きます。
3. `figfig config get popup`が`true`か確認します。
4. 一時停止中の場合は`figfig resume`を実行します。
5. `figfig permissions`からシステム設定を開き、figfigの権限を有効にします。
6. Terminal.appまたはiTerm2へ戻り、`git ch`の入力を確認します。

`figfig complete 'git ch'`で`candidates`が返る場合は、補完データが生成されています。ポップアップについては`doctor`の位置・権限情報も確認してください。

## ヘルパーが見つからない

ソースビルドでは次の両方があるか確認します。

```text
target/release/figfig
target/release/bundle/macos/figfig.app
```

ルートディレクトリで`pnpm run build:release`を実行すると、CLIとヘルパーをまとめて生成します。Homebrew配布では`bin/figfig`と`libexec/figfig.app`の配置を使います。

## 履歴予測が表示されない

`figfig config get history`を確認します。入力が空の場合、行末以外にカーソルがある場合、viコマンドモードでは履歴予測を表示しません。現在のzsh履歴に、入力中の文字列で始まるコマンドがあることも確認してください。

## キー操作やプロンプトが競合する

zsh連携をOh My Zshやプロンプトの初期化後に読み込みます。`zsh-autosuggestions`、`zsh-autocomplete`、Fig、Amazon Qなど、同じ入力やキーを扱う連携を確認してください。

切り分けには`figfig pause`を使えます。連携を外す手順は[インストールガイド](../getting-started/installation.md#連携を解除する)に記載しています。

## 設定ファイルのエラーが出る

`figfig config path`で場所を確認し、JSONの構文とキーを[設定ガイド](configuration.md#設定ファイル)の例に合わせます。CLIによる設定操作は、既存ファイルを読み込める状態で実行してください。

## ターミナルへ戻ると候補が消えている

フォーカスが別のアプリ・タブ・ペインへ移動したときや、入力位置の取得に失敗したときは候補を閉じます。対象のターミナルで入力内容を編集すると、新しい要求を送ります。

## 更新後に権限が認識されない

現在の配布物はad-hoc署名を使います。更新したバンドルで権限が必要になった場合は、`figfig permissions`で開くシステム設定からfigfigの登録状態を確認してください。

再現する問題を共有する場合は、macOS、ターミナル、zsh、figfigのバージョンと、最小の再現手順を添えてください。診断情報に含まれるホームディレクトリなどは、共有前に内容を確認できます。
