---
title: 設定
description: CLIからポップアップ、履歴予測、候補件数を設定する方法。
---

## 設定を表示する

```bash
figfig config
figfig config get popup
figfig config path
```

`config`と`config list`は現在の設定をJSONで表示します。`settings`も同じ操作に対応します。

## 変更できる項目

| キー | 値 | 初期値 | 内容 |
| --- | --- | --- | --- |
| `popup` | `true` / `false` | `true` | 補完ポップアップの表示 |
| `history` | `true` / `false` | `true` | zshの履歴に基づく予測 |
| `max-candidates` | 3〜12の整数 | `8` | 表示する候補の上限 |

```bash
figfig config set popup true
figfig config set history false
figfig config set max-candidates 6
```

設定を保存すると、稼働中のヘルパーが再読み込みし、接続中のシェルへ通知します。ヘルパーの停止中も設定ファイルを読み書きできます。

## 設定ファイル

初期設定では次の場所へ保存します。

```text
~/Library/Application Support/figfig/settings.json
```

JSON内ではキーが次の名前になります。

```json
{
  "popupEnabled": true,
  "historyEnabled": true,
  "maxCandidates": 8
}
```

CLIでは値を検証してから保存します。ファイルを直接編集した場合は、実行中のヘルパーを`figfig stop`で終了し、新しいzshセッションで読み込ませてください。

## 一時停止

```bash
figfig pause
figfig resume
```

一時停止の状態は保存用設定と別に管理します。ヘルパーを再起動すると通常の状態に戻ります。

## 開発・検証用の環境変数

| 変数 | 用途 |
| --- | --- |
| `FIGFIG_CONFIG_DIR` | `settings.json`を置くディレクトリ |
| `FIGFIG_RUNTIME_DIR` | Unixソケットを置くディレクトリ |
| `FIGFIG_APP` | CLIが起動するヘルパーのアプリバンドルまたは実行ファイル |
| `FIGFIG_TERMINAL_PID` | 通信ブリッジに渡すターミナルPIDの上書き |

場所を切り替える場合は、CLIとヘルパーが同じ環境を引き継ぐように起動してください。ランタイムディレクトリには、ユーザー所有でアクセス権0700のディレクトリを使います。
