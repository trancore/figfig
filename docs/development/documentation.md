---
title: ドキュメントサイトの運用
description: Docusaurusサイトの開発・検証とGitHub Pagesへの公開方法。
---

公開サイトはDocusaurusで構成し、日本語と英語に対応しています。日本語原稿は`docs/`、英語原稿とUI翻訳は`website/i18n/en/`、設定・トップページ・スタイルは`website/`にあります。

## ローカルで起動する

ルートディレクトリで実行します。

```bash
pnpm --dir website install --frozen-lockfile --ignore-scripts
pnpm run docs:dev
```

`http://localhost:3000/`を開きます。サイト用の依存関係は`website/pnpm-lock.yaml`で管理します。

## 言語ごとの確認

日本語を標準言語とし、英語版を`/en/`配下に公開します。ナビゲーションの言語メニューから、閲覧中のページに対応する翻訳へ切り替えられます。

開発サーバーは一度に一言語を起動します。英語版の開発では次を実行し、`http://localhost:3000/en/`を開きます。

```bash
pnpm run docs:dev --locale en
```

両言語の切替を含めて確認する場合は、全言語をビルドしてから`docs:serve`で配信します。

## 検証とビルド

```bash
pnpm run docs:check
pnpm run docs:build
pnpm run docs:serve
```

`docs:check`はTypeScriptを検査します。`docs:build`は両言語のMarkdown・サイト内リンクを検査し、日本語を`website/build/`、英語を`website/build/en/`へ生成します。`docs:serve`では生成済みのサイトを確認できます。

## GitHub Pagesへ公開する

1. このソースをGitHubリポジトリの`main`ブランチへ配置します。
2. リポジトリのSettings → Pagesを開き、Build and deploymentのSourceにGitHub Actionsを選びます。
3. `Documentation`ワークフローを手動実行するか、ドキュメントの変更を`main`へpushします。
4. ワークフローの`github-pages`環境に表示されるURLを開きます。

`.github/workflows/documentation.yml`はプルリクエストで型検査・静的ビルドを行い、`main`への変更と手動実行でPagesへデプロイします。公開ジョブではGitHubの`configure-pages`から取得したURLとパスを使います。

プロジェクトサイトの`/リポジトリ名/`と、ユーザー・組織サイトの`/`に対応します。カスタムドメインをPagesに設定した場合も、Pagesのメタデータからビルド先を決めます。公開元の設定手順はGitHubの公式ドキュメントを参照してください。[GitHub Pagesのカスタムワークフロー](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)

## 公開先を手動で指定する

| 環境変数 | 内容 | ローカルの初期値 |
| --- | --- | --- |
| `DOCS_URL` | 配信元のオリジン | `http://localhost:3000` |
| `DOCS_BASE_URL` | 先頭・末尾に`/`を含む公開パス | `/` |
| `GITHUB_REPOSITORY` | `OWNER/REPOSITORY`形式のリポジトリ名 | 未設定 |

`DOCS_URL`と`DOCS_BASE_URL`を省略し、`GITHUB_REPOSITORY`を設定した場合は、標準のGitHub Pages URLを使います。例の`example`は検証用の所有者名です。

```bash
GITHUB_REPOSITORY=example/figfig pnpm run docs:build
GITHUB_REPOSITORY=example/figfig pnpm run docs:serve
```

この場合、ローカルでは日本語の`http://localhost:3000/figfig/`と英語の`http://localhost:3000/figfig/en/`を確認します。公開前に、ナビゲーション・画像・直接アクセスが公開パスで機能することを確認してください。

## 原稿を追加する

`docs/`へ日本語Markdownを作成し、`website/i18n/en/docusaurus-plugin-content-docs/current/`の同じ相対パスに英語版を追加します。`website/sidebars.ts`へ文書IDを追加します。文書同士は相対パスでリンクします。既存のアーキテクチャ、検証記録、リリース手順も同じ原稿をサイトから表示します。

トップページの文言は`Translate`または`translate()`で定義し、英語訳を`website/i18n/en/code.json`へ記入します。ナビゲーションとフッターは`docusaurus-theme-classic/`、サイドバーのカテゴリ名は`docusaurus-plugin-content-docs/current.json`で翻訳します。

新しいUI文言は次のコマンドで抽出できます。抽出後は追加されたメッセージを翻訳してください。

```bash
pnpm --dir website exec docusaurus write-translations --locale en
```

英語の見出しには日本語原稿と同じIDを`{#見出しID}`で明示し、言語を切り替えたときも同じセクションへ移動できるようにします。文書内リンクにもそのIDを使います。言語・公開パスの詳細は公式ドキュメントを参照してください。[Docusaurusの多言語対応](https://docusaurus.io/docs/i18n/tutorial)

Docusaurusの設定・公開パスの詳細は公式ドキュメントを参照してください。[Docusaurusのデプロイ](https://docusaurus.io/docs/deployment)
