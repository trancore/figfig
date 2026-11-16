import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  documentation: [
    {
      type: 'category', label: 'はじめに', collapsed: false,
      items: ['getting-started/installation', 'guides/usage', 'guides/configuration'],
    },
    {
      type: 'category', label: 'リファレンス', collapsed: false,
      items: ['reference/cli', 'reference/compatibility', 'reference/privacy', 'guides/troubleshooting'],
    },
    {
      type: 'category', label: '開発', collapsed: false,
      items: ['development/build', 'development/specifications', 'architecture', 'verification', 'releasing', 'development/documentation'],
    },
  ],
};

export default sidebars;
