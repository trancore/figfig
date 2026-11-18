---
title: 対応環境
description: OS・ターミナル・構文の対応範囲と、実機確認の状況。
---

## 初期版の対象

| 項目 | 対応範囲 |
| --- | --- |
| OS | macOS 13以降 |
| シェル | zsh 5.9以降 |
| ターミナル | Terminal.app、iTerm2、VSCode・Cursor・Zed・主要JetBrains IDEの内蔵ターミナル |
| シェルのキー設定 | emacs、viの挿入モード |
| 候補生成 | ローカルのコマンド定義、PATH、ファイル、Gitブランチ、package scripts |

SSH先、tmux内、他のシェル・ターミナルは初期版の対象範囲外です。

## 内蔵ターミナルの要件

| ホスト | 位置・フォーカス取得 | 検証状況 |
| --- | --- | --- |
| VSCode stable／Insiders、Cursor | Accessibility、必要時は共通InputMethodKitヘルパー | 識別・ネイティブ座標・PTY祖先判定を自動検証。実ポップアップの受け入れ確認は未完了 |
| Zed stable／Preview／Nightly | InputMethodKit（`figfig setup terminal`） | 識別・ネイティブ座標・通信・祖先判定を自動検証。実ポップアップの受け入れ確認は未完了 |
| IntelliJ IDEA、PyCharm、WebStorm、GoLand、CLion、Rider、RubyMine、DataGrip、RustRover、Android Studio | figfig Terminalプラグイン | 導入済みIntelliJ SDKでのビルドと実Java通信を検証。製品ごとの実機確認は未完了 |

JetBrainsはClassicとReworkedを対象とし、廃止済みExperimental 2024は含めません。識別への対応と、全バージョン・ターミナル方式での実機確認は区別します。[導入手順](../getting-started/installation.md#zedの内蔵ターミナルで使う)も参照してください。

## 入力の解析

空白、引用符、Unicodeを含むリテラルの入力と、パイプ・コマンド区切り後の入力に対応します。ファイル候補は、空白やシェル記号を含む名前を引用して挿入します。

コマンド置換、変数展開、リダイレクトなどの構文は解析対象の範囲外です。候補を生成できない入力では、元のzshのキー操作を使用できます。

## コマンド定義

git、brew、npm、pnpm、dockerの主要なサブコマンドとオプションを定義しています。Gitの動的候補はローカルブランチ、パッケージマネージャーの動的候補は作業ディレクトリのpackage scriptsです。

独自コマンドの定義方法は[コマンド定義の追加](../development/specifications.md)に記載しています。

## 権限・フォーカス

Terminal.app／iTerm2はmacOS Accessibilityから入力位置を取得します。VSCode／Cursorはフォーカスされたxterm入力欄を識別してAccessibility座標を優先し、必要時は共通InputMethodKitヘルパーを使います。対応するターミナルが最前面にあり、シェルのターミナルPIDと一致する場合にポップアップを表示します。

ZedはInputMethodKit、JetBrains系は専用プラグインでカーソル座標を取得します。別の位置やペインへ移動した場合は候補を閉じ、新しいシェル入力を待ちます。位置取得に失敗した場合やヘルパー未登録時も候補を閉じ、`figfig doctor`に診断を表示します。

## 検証状況

内蔵ターミナル連携の座標変換、古い応答の破棄、ネイティブヘルパーの通信、エディタの祖先PID、セットアップ互換コマンド、JetBrainsプラグインの実Java通信は自動テストで検証します。実際の表示・IME・ペイン切り替えは別途実機確認が必要です。

Apple Silicon・macOSで、自動テスト、本番ビルド、署名、実ヘルパーとの通信を確認しています。Terminal.appとiTerm2での基本表示はCLI化前の版で手動確認済みです。

Homebrew配布は公開準備中です。Apple Silicon・macOS 27.0.1では、ローカルの検証専用tapからソースビルド・インストール・Formulaの監査・テスト・削除を確認しました。公開Releaseと正式tapからの導入は、公開後に確認します。手順は[リリース手順](../releasing.md)を参照してください。

Intelでのビルド、IME、画面倍率の異なる複数画面、フルスクリーンなどの詳細操作と、公開tapからの導入・更新・削除は未確認です。具体的な結果と確認手順は[検証記録](../verification.md)を参照してください。
