---
title: コマンド一覧
description: figfig CLIのコマンド、引数、設定キー、終了コード。
---

## 導入・連携

| コマンド | 動作 |
| --- | --- |
| `figfig setup` | `.zshrc`へ連携を追加。既存ファイルのバックアップを作成 |
| `figfig init zsh` | zsh連携スクリプトを標準出力へ出力 |
| `figfig permissions` | ヘルパーを起動し、macOSの権限を確認・要求 |

[導入ガイド](../getting-started/installation.md)に手順を記載しています。

## 設定

```bash
figfig config
figfig config list
figfig config path
figfig config get KEY
figfig config set KEY VALUE
```

| キー | 指定できる値 |
| --- | --- |
| `popup` | `true` / `false` |
| `history` | `true` / `false` |
| `max-candidates` | 3〜12 |

`figfig settings`は`figfig config`と同じ引数で利用できます。保存先と初期値は[設定ガイド](../guides/configuration.md)を参照してください。

## 実行状態

| コマンド | 動作 |
| --- | --- |
| `figfig start` | バックグラウンドの描画ヘルパーを起動。稼働中はその接続を使用 |
| `figfig stop` | ヘルパーを終了 |
| `figfig pause` | ヘルパーを起動し、候補表示・履歴予測を一時停止 |
| `figfig resume` | ヘルパーを起動し、候補表示・履歴予測を再開 |
| `figfig doctor` | 接続・権限・位置取得の診断 |

通常のzsh連携ではヘルパーを自動起動します。`stop`の後も、新しいzshセッションでは再起動できます。

## 補完結果を調べる

```bash
figfig complete TEXT [DIRECTORY]
```

入力文字列の末尾をカーソル位置として、上限12件の候補をJSONで出力します。ディレクトリを省略すると現在の作業ディレクトリを使います。

```bash
figfig complete 'git status --s'
figfig complete 'pnpm run ' /path/to/project
```

応答の`replaceStart`と`replaceEnd`はUnicodeスカラー数による置換範囲です。通信型の詳細は[アーキテクチャ](../architecture.md)を参照してください。

## ヘルプとバージョン

```bash
figfig --help
figfig -h
figfig --version
figfig -V
```

引数を省略した場合もヘルプを表示します。

## 内部のシェル通信

`figfig bridge SESSION_ID`はzsh連携が起動する常駐通信ブリッジです。NUL区切りの入力を受け取り、Unixソケットを通じてヘルパーと通信します。通常の利用は`setup`または`init zsh`から連携します。

## 終了コード

通常終了は`0`、引数の不備・設定エラー・接続失敗などは`1`です。`doctor`はアクセシビリティ権限が不足している場合も`1`を返します。
