import type { ReactNode } from 'react';
import Link from '@docusaurus/Link';
import Translate, { translate } from '@docusaurus/Translate';
import Layout from '@theme/Layout';
import styles from './index.module.css';

export default function Home(): ReactNode {
  return (
    <Layout
      title={translate({ id: 'homepage.title', message: 'ターミナル補完と履歴予測' })}
      description={translate({
        id: 'homepage.description',
        message: 'figfigはmacOSのzshに補完ポップアップと履歴予測を追加するCLIツールです。導入からコマンド一覧、設定、開発手順までを案内します。',
      })}
    >
      <main className={styles.home}>
        <section className={styles.hero} aria-labelledby="hero-title">
          <div>
            <p className={styles.compatibility}>macOS · zsh · Terminal.app / iTerm2 / VSCode / Cursor / Zed / JetBrains</p>
            <h1 id="hero-title">
              <Translate id="homepage.heading.first">ターミナルに、</Translate><br />
              <Translate id="homepage.heading.second">補完と履歴予測を。</Translate>
            </h1>
            <p className={styles.introduction}>
              <Translate id="homepage.introduction">
                figfigは入力中のコマンドやパスを補完するCLIツールです。導入も設定もコマンドから。いつものターミナルで使い始められます。
              </Translate>
            </p>
            <div className={styles.actions}>
              <Link className="button button--primary button--lg" to="/docs/getting-started/installation">
                <Translate id="homepage.install">導入ガイド</Translate>
              </Link>
              <Link className="button button--outline button--primary button--lg" to="/docs/reference/cli">
                <Translate id="homepage.commands">コマンド一覧</Translate>
              </Link>
            </div>
            <p className={styles.releaseNote}>
              <Translate id="homepage.releaseNote">Homebrew tap（trancore/tap）は公開準備中です。現在はソースから導入できます。</Translate>
            </p>
          </div>
          <figure className={styles.terminal}>
            <figcaption><Translate id="homepage.demo.caption">補完ポップアップの表示例</Translate></figcaption>
            <div className={styles.terminalContent}>
              <p className={styles.prompt}><span>~/projects</span> $ git ch<span className={styles.cursor} aria-hidden="true">▏</span></p>
              <div className={styles.popup}>
                <ul className={styles.popupCandidates}>
                  {['checkout', 'cherry-pick'].map((candidate, index) => (
                    <li key={candidate} className={styles.candidate} data-selected={index === 0}>
                      <svg className={styles.candidateIcon} aria-hidden="true" focusable="false" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
                        <path d="M6 4v8a3 3 0 0 0 3 3h10m-4-4 4 4-4 4" />
                      </svg>
                      <span className={styles.candidateLabel}>{candidate}</span>
                    </li>
                  ))}
                </ul>
                <div className={styles.popupFooter}>
                  <span className={styles.popupBrand} aria-hidden="true"><span>fig</span><span>fig</span></span>
                  <span className={styles.popupDescription}>Switch branches or restore files</span>
                  <span className={styles.popupHint} aria-label={translate({ id: 'homepage.demo.accept', message: 'Tab または Enter で挿入' })}>
                    <kbd>tab</kbd><kbd>↵</kbd>
                  </span>
                </div>
              </div>
              <p className={styles.demoNote}>
                <Translate id="homepage.demo.note">候補は種類ごとに色分けされ、選択中の候補の説明が下部に表示されます。</Translate>
              </p>
            </div>
          </figure>
        </section>
        <nav
          className={styles.guides}
          aria-label={translate({ id: 'homepage.guides.label', message: '目的からドキュメントを探す' })}
        >
          <Link className={styles.guide} to="/docs/guides/usage">
            <h2><Translate id="homepage.usage.title">キー操作を覚える</Translate> <span aria-hidden="true">↗</span></h2>
            <p><Translate id="homepage.usage.description">候補の選択・挿入と、履歴予測の使い方。</Translate></p>
          </Link>
          <Link className={styles.guide} to="/docs/guides/configuration">
            <h2><Translate id="homepage.configuration.title">自分の環境に合わせる</Translate> <span aria-hidden="true">↗</span></h2>
            <p><Translate id="homepage.configuration.description">ポップアップ、履歴予測、表示件数の設定。</Translate></p>
          </Link>
          <Link className={styles.guide} to="/docs/guides/troubleshooting">
            <h2><Translate id="homepage.troubleshooting.title">候補が表示されないとき</Translate> <span aria-hidden="true">↗</span></h2>
            <p><Translate id="homepage.troubleshooting.description">起動状態・権限・シェル連携を確認する。</Translate></p>
          </Link>
        </nav>
      </main>
    </Layout>
  );
}
