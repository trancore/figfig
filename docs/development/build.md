---
title: 開発環境とビルド
description: TypeScript・Rustの開発環境、ビルド、テストの実行手順。
---

## 必要な環境

macOS上でNode.js 24.15以降の24系、または26以降、pnpm 12.21.0、Rust 1.90以降、Xcode Command Line Toolsを使います。取得済みのソースのルートディレクトリで実行してください。

```bash
pnpm install --frozen-lockfile --ignore-scripts
```

依存関係は`pnpm-lock.yaml`と`Cargo.lock`で管理しています。Rustの開発ツールチェーンは`rust-toolchain.toml`で1.100.0を指定し、rustupがプロジェクト内で選択します。

依存ライブラリは最新の安定版を基本に選び、Docusaurus関連パッケージは同じバージョンで揃えます。2026-11-16時点のDocusaurus 3.10.2のMermaidテーマはELK 0.1系を要求するため、図表示にはMermaid 11.17.2と`@mermaid-js/layout-elk` 0.1.9を使用します。

## 開発時の起動

```bash
pnpm run dev
```

Reactの表示部分だけを確認する場合は次を実行し、`http://localhost:1420/?preview=1`を開きます。

```bash
pnpm run dev:ui
```

## 本番ビルド

```bash
pnpm run build:release
```

| 成果物 | 場所 |
| --- | --- |
| CLI | `target/release/figfig` |
| 描画ヘルパー | `target/release/bundle/macos/figfig.app` |

macOSのad-hoc署名を使います。CLIが同じ配置のヘルパーを自動で起動します。

## 自動テスト

```bash
pnpm run check
pnpm test
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build -p figfig-cli --locked
pnpm run test:shell
```

`test:shell`はビルド済みのデバッグCLIを使用します。zsh、PTY、テスト専用ヘルパー、一時的なHOME・設定・Unixソケットを使って連携を検証します。

本番ビルド済みの実ヘルパーとの通信は、既存のfigfigヘルパーを終了してから確認できます。

```bash
python3 scripts/smoke-cli.py
```

この検証にはディスプレイのあるmacOS環境を使います。画面上の位置やIMEなどの確認項目は[検証記録](../verification.md)を参照してください。

## ソースの構成

| 場所 | 責務 |
| --- | --- |
| `crates/core` | 入力解析、候補生成、通信型、設定、セッション状態 |
| `crates/cli` | コマンド処理、起動管理、zsh初期設定、シェル通信 |
| `src-tauri` | 接続処理、非同期候補生成、macOSの位置取得、表示管理 |
| `src` | React製ポップアップ |
| `shell/figfig.zsh` | ZLEの入力・キー操作・履歴予測 |
| `specs/index.ts` | 組み込みコマンド定義 |
| `website` | Docusaurusによる公開ドキュメントのサイト設定・表示 |
| `docs` | 公開ドキュメントの日本語Markdown原稿 |
| `website/i18n/en/docusaurus-plugin-content-docs/current` | 公開ドキュメントの英語Markdown原稿 |

配布の流れは[リリース手順](../releasing.md)、サイトの開発・公開は[ドキュメントサイトの運用](documentation.md)を参照してください。
