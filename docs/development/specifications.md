---
title: コマンド定義の追加
description: TypeScriptでサブコマンド・オプションを定義し、Rust用データを生成する方法。
---

組み込み候補は`specs/index.ts`の`specs`配列へ追加します。型は`CommandSpec`、`SubcommandSpec`、`OptionSpec`です。

## 定義の例

次のオブジェクトを`specs`配列へ追加すると、`example-cli deploy`とそのオプションを定義できます。

```typescript
{
  name: "example-cli",
  description: "Example command",
  options: [
    { name: "--help", description: "Show help" },
  ],
  subcommands: [
    {
      name: "deploy",
      description: "Deploy the project",
      options: [
        {
          name: "--target",
          description: "Select a target",
          takesValue: true,
        },
      ],
    },
  ],
}
```

`name`は補完へ挿入する文字列、`description`は表示する説明です。値を取るオプションには`takesValue: true`を付けます。定義からコマンドを実行する処理は発生しません。

## 動的候補

サブコマンドの`generator`には次の値を指定できます。

| 値 | 候補 |
| --- | --- |
| `gitBranches` | 現在のリポジトリのローカルブランチ |
| `packageScripts` | 作業ディレクトリの`package.json`の`scripts` |

新しい生成方式を追加する場合は、TypeScriptの型とRust側の`crates/core/src/generators.rs`・`engine.rs`を更新します。

## Rust用のデータを更新する

```bash
pnpm run specs
pnpm run check
cargo test -p figfig-core --locked
```

`assets/specs.json`が生成物です。TypeScriptの定義と一緒に管理してください。`pnpm run specs:check`では一致を検証します。

CLIで実際の候補を確認できます。

```bash
cargo build -p figfig-cli --locked
./target/debug/figfig complete 'example-cli de'
```
