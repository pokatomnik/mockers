# `mockers completion` Command Reference

## Overview

The `completion` command generates a shell completion script for the `mockers` CLI and prints it to standard output.

- Primary goal: improve CLI ergonomics by enabling tab completion for commands and options.
- Implementation: uses `clap_complete::generate` against the full `Cli` command tree.

---

## Command Purpose

`mockers completion` is intended to:

1. Generate auto-completion scripts for supported shells.
2. Keep completions aligned with the current CLI structure (`serve`, `create`, `list`, `info`, `delete`, `enable`, `disable`, `config`, `init`, `completion`).
3. Provide shell-specific completion output that can be sourced or installed into shell startup/completion directories.

---

## Syntax

```bash
mockers completion [OPTIONS]
```

### Options

| Option | Short | Default | Values | Description |
|---|---|---|---|---|
| `--shell <SHELL>` | `-s` | `bash` | `bash`, `elvish`, `fish`, `powershell`, `zsh` | Selects which shell completion script to generate |
| `--help` | `-h` | — | — | Prints command help |

---

## How It Works

At runtime, the command:

1. Builds a `clap` command model from `Cli::command()`.
2. Reads the command name from the model (`mockers`).
3. Calls `clap_complete::generate(...)` with the selected shell.
4. Writes the resulting script to `stdout`.

### Practical implications

- The output is **not** written to a file automatically.
- You should redirect output to a file or evaluate/source it directly in your shell, example:
- Completion definitions stay synchronized with the actual compiled binary and command metadata.

---

## Configuration and Behavior Details

### Runtime configuration

The command itself has only one functional parameter: `--shell`.

There are no additional config file settings for completion generation in `mockers` global configuration. The command is deterministic for a given binary version and shell target.

### Error behavior

- In normal flow, generation returns `Ok(())`.
- Errors are propagated through the common CLI error handling path in `main`.

---

## Examples

### Print Bash completion script

```bash
mockers completion
```

### Print Zsh completion script

```bash
mockers completion --shell zsh
```

### Save script to a file

```bash
mockers completion --shell fish > ~/.config/fish/completions/mockers.fish
```

### Load for current shell session (example pattern)

```bash
eval "$(mockers completion --shell bash)"
```

> Note: exact installation paths/commands differ by shell and OS distribution.

---

### Make completitions peristent

`mockers` can generate shell completion scripts for supported shells.  
To enable completions, add the following line to your shell startup file:

```sh
source <(mockers completion -s SHELL)
```

---

## Notes for Documentation Consistency

- The `completion` command is implemented in code and available in CLI help.
- At the time of analysis, README command overview does not document `completion`; consider adding it to avoid discoverability gaps.
