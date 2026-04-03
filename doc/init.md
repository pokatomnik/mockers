# `mockers init` Command Reference

## Purpose

`mockers init` initializes the **global Mockers configuration file** (`.mockers`) in the current user's home directory.

This file stores default values that are later used by commands such as `serve`, `create`, `list`, `info`, `delete`, `enable`, and `disable` through the shared global config loader.

---

## Syntax

```bash
mockers init [--interactive|-i]
```

Aliases:

- `mockers init`
- `mockers setup`

---

## Parameters

| Flag | Short | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `--interactive` | `-i` | boolean flag | `false` | Enables interactive setup (prompts in terminal). |

If `--interactive` is **not** specified, the command writes a config with built-in defaults.

---

## Output file and location

- File name: `.mockers`
- Location: `$HOME/.mockers`
- Format: JSON (`camelCase` keys)

The command resolves the home directory via `std::env::home_dir()` and writes the generated JSON there.

---

## Non-interactive mode (default)

When run without `--interactive`, the command writes a "fair defaults" configuration:

```json
{
  "host": "0.0.0.0",
  "port": 8080,
  "httpsPort": 8443,
  "mocks": "mocks",
  "cors": false,
  "preflight": "permissive",
  "delayMs": 0,
  "adminBaseUrl": "/__admin",
  "logRequest": "info",
  "verbosity": "info",
  "proxyBodyMaxBytes": 16777216
}
```

Notes:

- `origin` is not set by default.

---

## Interactive mode (`--interactive`)

Interactive mode asks for each field step by step and builds the same global config structure.

Prompts include:

1. Host/IP
2. Port
3. HTTPS Port
4. Mocks directory
5. CORS on/off
6. Preflight mode (`permissive` or `mirror`)
7. Global delay (ms)
8. Admin base URL
9. Request log level (`info`, `debug`, `trace`)
10. Verbosity level (`info`, `debug`, `trace`)

Prompt behavior details:

- Each prompt provides a default value.
- If input parsing fails, the corresponding default is used.
- Port validation expects a `u16` number.

---

## Overwrite behavior

If `$HOME/.mockers` already exists (file, directory, or symlink), `init` asks for confirmation before writing:

- **Yes** -> overwrite with new config.
- **No** -> command exits without changes.

If the file does not exist, it writes immediately.

---

## Configuration precedence in runtime

The global config system can merge multiple `.mockers` files discovered from filesystem hierarchy (from root to current directory).

For each field, later (closer) config values override earlier ones. CLI arguments still have higher priority where applicable.

For example in `serve`:

1. CLI flags (`--host`, `--port`, etc.)
2. Merged global config (`.mockers`)
3. Built-in hard defaults

---

## Related commands

- `mockers config` — prints effective global config values.
- `mockers serve` — consumes global config as defaults unless flags override.

---

## Practical examples

### 1) Create default global config

```bash
mockers init
```

### 2) Run interactive setup

```bash
mockers init --interactive
```

### 3) Use alias

```bash
mockers setup -i
```
