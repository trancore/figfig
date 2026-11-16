---
title: Adding command definitions
description: Define subcommands and options in TypeScript and generate the data consumed by Rust.
---

Add built-in candidates to the `specs` array in `specs/index.ts`. The interfaces are `CommandSpec`, `SubcommandSpec`, and `OptionSpec`.

## Example definition {#定義の例}

Add the following object to `specs` to define `example-cli deploy` and its options.

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

`name` is the text inserted by completion, and `description` is shown in the popup. Set `takesValue: true` for options that consume a value. These definitions supply completion data without executing commands.

## Dynamic candidates {#動的候補}

A subcommand's `generator` can use either of the following values.

| Value | Candidates |
| --- | --- |
| `gitBranches` | Local branches in the current repository |
| `packageScripts` | Entries in `scripts` from the working directory's `package.json` |

To add a new generator, update the TypeScript type and the Rust implementation in `crates/core/src/generators.rs` and `engine.rs`.

## Regenerate Rust data {#rust用のデータを更新する}

```bash
pnpm run specs
pnpm run check
cargo test -p figfig-core --locked
```

The generated file is `assets/specs.json`. Include it with changes to the TypeScript definitions. `pnpm run specs:check` verifies that the two match.

Inspect the resulting candidates through the CLI.

```bash
cargo build -p figfig-cli --locked
./target/debug/figfig complete 'example-cli de'
```
