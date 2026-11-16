import type { Config } from '@docusaurus/types';
import type { Options, ThemeConfig } from '@docusaurus/preset-classic';
import { themes } from 'prism-react-renderer';

const repository = process.env.GITHUB_REPOSITORY;
const [owner, name] = repository?.split('/') ?? [];
const isUserSite = name?.toLowerCase() === `${owner?.toLowerCase()}.github.io`;
const config: Config = {
  title: 'figfig',
  tagline: 'zshの補完と履歴予測を、いつものターミナルに。',
  favicon: 'img/icon.svg',
  url: process.env.DOCS_URL || (owner ? `https://${owner}.github.io` : 'http://localhost:3000'),
  baseUrl: process.env.DOCS_BASE_URL || (name && !isUserSite ? `/${name}/` : '/'),
  trailingSlash: true,
  onBrokenLinks: 'throw',
  onBrokenAnchors: 'throw',
  i18n: {
    defaultLocale: 'ja',
    locales: ['ja', 'en'],
    localeConfigs: {
      ja: { label: '日本語', htmlLang: 'ja' },
      en: { label: 'English', htmlLang: 'en' },
    },
  },
  markdown: {
    mermaid: true,
    hooks: { onBrokenMarkdownLinks: 'throw' },
  },
  themes: ['@docusaurus/theme-mermaid'],
  presets: [
    ['classic', {
      docs: { path: '../docs', sidebarPath: './sidebars.ts' },
      blog: false,
      theme: { customCss: './src/css/custom.css' },
    } satisfies Options],
  ],
  themeConfig: {
    colorMode: { respectPrefersColorScheme: true },
    navbar: {
      title: 'figfig',
      logo: { alt: '', src: 'img/icon.svg', width: 30, height: 30 },
      items: [
        { type: 'docSidebar', sidebarId: 'documentation', label: 'ドキュメント', position: 'left' },
        { to: '/docs/reference/cli', label: 'コマンド一覧', position: 'left' },
        { type: 'localeDropdown', position: 'right' },
        ...(repository ? [{ href: `https://github.com/${repository}`, label: 'GitHub', position: 'right' as const }] : []),
      ],
    },
    footer: {
      style: 'dark',
      links: [
        { title: '使い始める', items: [
          { label: 'インストール', to: '/docs/getting-started/installation' },
          { label: '基本操作', to: '/docs/guides/usage' },
          { label: '設定', to: '/docs/guides/configuration' },
        ] },
        { title: '調べる', items: [
          { label: 'コマンド一覧', to: '/docs/reference/cli' },
          { label: 'トラブルシューティング', to: '/docs/guides/troubleshooting' },
          { label: 'プライバシー', to: '/docs/reference/privacy' },
        ] },
        { title: '開発に参加する', items: [
          { label: '開発環境', to: '/docs/development/build' },
          { label: 'コマンド定義の追加', to: '/docs/development/specifications' },
          { label: '検証状況', to: '/docs/verification' },
        ] },
      ],
      copyright: 'figfig · MIT License',
    },
    prism: { theme: themes.github, darkTheme: themes.dracula, additionalLanguages: ['bash', 'rust', 'toml', 'json'] },
  } satisfies ThemeConfig,
};

export default config;
