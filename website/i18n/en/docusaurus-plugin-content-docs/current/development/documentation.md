---
title: Documentation site
description: Develop and translate the Docusaurus website, verify both languages, and publish to GitHub Pages.
---

The public website uses Docusaurus with Japanese and English content. Japanese source lives in `docs/`, English pages and UI translations live in `website/i18n/en/`, and site configuration, the homepage, and styles live in `website/`.

## Start locally {#ローカルで起動する}

Run the following commands from the repository root.

```bash
pnpm --dir website install --frozen-lockfile --ignore-scripts
pnpm run docs:dev
```

Open `http://localhost:3000/`. Website dependencies are recorded in `website/pnpm-lock.yaml`.

## Preview each language {#言語ごとの確認}

Japanese is the default language. English is published under `/en/`. The language menu links to the corresponding translation of the page you are viewing.

The development server runs one language at a time. To work on English content, run the following command and open `http://localhost:3000/en/`.

```bash
pnpm run docs:dev --locale en
```

To test switching between both languages, build all locales and use `docs:serve`.

## Check and build {#検証とビルド}

```bash
pnpm run docs:check
pnpm run docs:build
pnpm run docs:serve
```

`docs:check` validates TypeScript. `docs:build` checks Markdown and internal links in both languages. Japanese output is generated in `website/build/` and English output in `website/build/en/`. `docs:serve` previews the generated site.

## Publish to GitHub Pages {#github-pagesへ公開する}

1. Place the source in the `main` branch of a GitHub repository.
2. Open Settings → Pages, then select GitHub Actions as the Source under Build and deployment.
3. Run the `Documentation` workflow manually or push documentation changes to `main`.
4. Open the URL shown for the workflow's `github-pages` environment.

`.github/workflows/documentation.yml` checks types and builds the site for pull requests. Changes on `main` and manual runs on that branch deploy to Pages. The publishing job reads the public origin and base path from GitHub's `configure-pages` action.

Both project sites under `/repository-name/` and user or organization sites at `/` are supported. A custom domain configured in Pages is also read from its metadata. See GitHub's official instructions for configuring the publishing source. [Custom workflows with GitHub Pages](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)

## Override the publishing location {#公開先を手動で指定する}

| Variable | Purpose | Local default |
| --- | --- | --- |
| `DOCS_URL` | Public origin | `http://localhost:3000` |
| `DOCS_BASE_URL` | Public path with leading and trailing `/` | `/` |
| `GITHUB_REPOSITORY` | Repository in `OWNER/REPOSITORY` form | Unset |

When `DOCS_URL` and `DOCS_BASE_URL` are omitted and `GITHUB_REPOSITORY` is set, the configuration uses the standard GitHub Pages URL. The `example` owner below is for local verification.

```bash
GITHUB_REPOSITORY=example/figfig pnpm run docs:build
GITHUB_REPOSITORY=example/figfig pnpm run docs:serve
```

In this example, preview Japanese at `http://localhost:3000/figfig/` and English at `http://localhost:3000/figfig/en/`. Verify navigation, images, and direct links at the publishing path before deployment.

## Add and translate content {#原稿を追加する}

Create Japanese Markdown in `docs/` and add its English counterpart at the same relative path under `website/i18n/en/docusaurus-plugin-content-docs/current/`. Add the document ID to `website/sidebars.ts`. Use relative paths for links between documents. Architecture, verification, and release pages use the same documentation sources as the rest of the site.

Define homepage messages with `Translate` or `translate()` and put English translations in `website/i18n/en/code.json`. Navbar and footer labels use `docusaurus-theme-classic/`. Sidebar categories use `docusaurus-plugin-content-docs/current.json`.

Extract new UI messages with the following command, then translate the added entries.

```bash
pnpm --dir website exec docusaurus write-translations --locale en
```

Give translated headings explicit IDs using the Japanese source heading IDs. This lets the language switcher preserve the current section. Use those same IDs in links between documents. See the official guide for locale and routing behavior. [Docusaurus internationalization](https://docusaurus.io/docs/i18n/tutorial)

For publishing configuration, see the official deployment guide. [Docusaurus deployment](https://docusaurus.io/docs/deployment)
