# figfig documentation

英語・日本語の公開ドキュメントをDocusaurusで生成します。Node.js 24.15以降の24系、または26以降、pnpm 12.21.0を使います。

ルートディレクトリで実行します。

```bash
pnpm --dir website install --frozen-lockfile --ignore-scripts
pnpm run docs:dev
pnpm run docs:check
pnpm run docs:build
pnpm run docs:serve
```

日本語の開発URLは`http://localhost:3000/`です。英語は`pnpm run docs:dev --locale en`で起動し、`http://localhost:3000/en/`を開きます。両言語をビルドして`docs:serve`で配信すると、言語メニューで切り替えられます。

日本語原稿は`docs/`、英語原稿は`website/i18n/en/docusaurus-plugin-content-docs/current/`で編集します。トップページの英語訳は`website/i18n/en/code.json`、ナビゲーションは`website/sidebars.ts`と`website/i18n/en/`内のJSONで管理します。生成先は`website/build/`です。

GitHub Pagesの有効化、公開URL・サブパスの指定は[ドキュメントサイトの運用](../docs/development/documentation.md)を参照してください。
