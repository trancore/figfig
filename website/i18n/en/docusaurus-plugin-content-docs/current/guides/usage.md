---
title: Basic usage
description: Select and insert completions, accept history suggestions, and understand candidate sources.
---

## Select and insert a suggestion {#候補を選んで挿入する}

As you type a command, suggestions and descriptions appear near the cursor. Keyboard focus stays in the terminal.

Each candidate occupies one row, with its label and icon colored by type. A tinted background and a left-edge marker identify the selected row. Its description appears in the footer. If a description is missing, the footer shows the candidate type, using `folder` for directories. Long labels and descriptions are truncated.

The popup uses a fixed dark palette. Icon shapes also identify candidate types, so color is not the only distinction.

| Key | Action |
| --- | --- |
| ↑ / ↓ | Select a candidate in the popup |
| Tab / Enter | Insert the selected candidate |
| Esc | Close the popup |
| → at the end of the line | Accept the visible history suggestion |

After Enter inserts a candidate, press Enter again to execute the command. Following insertion or dismissal with Esc, suggestions appear again when you edit the input. Your original zsh bindings handle keys when the popup is inactive.

## Completion sources {#補完できる内容}

| Type | Color | Example input | Source |
| --- | --- | --- | --- |
| Command names | Cyan | `gi` | Built-in definitions, executables on PATH, shell commands |
| Subcommands | Mint | `git ch` | Built-in git, brew, npm, pnpm, and docker definitions |
| Options | Yellow | `git status --s` | Command and subcommand definitions |
| Directories | Blue | `cd src/` | The working directory and the path being typed |
| Files | Purple | `cat READ` | The working directory and the path being typed |
| Git branches | Pink | `git switch ` | Local branches in the current repository |
| Package scripts | Orange | `pnpm run ` | `scripts` in the working directory's `package.json` |

Package names, Docker container names, and loading arbitrary Fig specifications or zsh completion functions are outside the current built-in scope. See [Compatibility](../reference/compatibility.md) for details.

## Use history suggestions {#履歴予測を使う}

figfig searches the current zsh history for commands that start with your input. When the remaining text appears at the end of the line, press → to accept it.

Toggle history suggestions with the following commands.

```bash
figfig config set history false
figfig config set history true
```

History lookup and display run inside zsh. Your zsh settings control history-file storage.

## Pause and resume {#一時停止と再開}

```bash
figfig pause
figfig resume
```

Pause lasts until the current helper exits. For persistent preferences, see [Configuration](configuration.md).

## Inspect suggestions as JSON {#候補をjsonで調べる}

```bash
figfig complete 'git ch'
figfig complete 'pnpm run ' .
```

`complete` prints a response from the local completion engine. The `visible` field is `false` for this command. Inspect `candidates` to see the generated suggestions.
