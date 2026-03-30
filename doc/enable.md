# `mockers enable` Command Reference

## Purpose

`mockers enable` re-activates a mock that was previously disabled.

Internally, the command updates the `disabled` flag for a single mock entry in a nearby `config.json` file:

- `disable` writes `disabled: true`
- `enable` writes `disabled: false`

At serve time, disabled mocks are skipped, while enabled mocks can be served normally.

---

## Syntax

```bash
mockers enable [OPTIONS] <NAME>
```

Where:

- `<NAME>` — required search string (full or partial), used to find a mock file by path/name (case-insensitive).

---

## Parameters

### Positional

- `NAME` (required)
  - Accepts a full or partial mock path/file name.
  - Match is case-insensitive.

### Options

- `-m, --mocks <MOCKS_PATH>` (optional)
  - Path to directory containing mock files.
  - If omitted, the command resolves mocks directory in this order:
    1. CLI `--mocks`
    2. global `.mockers` config (`mocks`)
    3. default `<current_dir>/mocks`

### Aliases

- `enable` has CLI alias: `on`.

---

## How matching works

The command scans the resolved mocks directory recursively and keeps only files whose extension is a valid HTTP method (`get`, `post`, `put`, etc.).

Then it applies a case-insensitive substring match against the full file path using `<NAME>`.

### Outcomes

1. **No matches**  
   Prints:
   - `There are no mocks matching your query`

2. **More than one match**  
   Prints the candidate list and asks for a more specific query.

3. **Exactly one match**  
   Updates that mock's `disabled` status in `config.json` to `false`.

---

## Configuration behavior (`config.json`)

For the selected mock:

- Entry key is the mock file name (e.g., `users.get`).
- Config file path is sibling directory of the mock file plus `config.json`.
- If entry already exists, fields are preserved and only `disabled` is changed.
- If entry is missing, a default config is created and then `disabled: false` is applied.
- If `config.json` is absent, it is created automatically.

---

## Runtime effect in `serve`

`mockers serve` checks the same `config.json` and reads `disabled` for the matched mock entry.

- If `disabled == true`, local mock file is ignored.
- If `disabled == false`, local mock file may be returned (with its configured status/headers/delay).

So, `mockers enable` effectively allows the mock to participate in normal file-based response flow again.

---

## Examples

Enable by full file name:

```bash
mockers enable users.get
```

Enable by partial path:

```bash
mockers enable users
```

Enable using custom mocks directory:

```bash
mockers enable --mocks ./fixtures/mocks users
```

Using alias:

```bash
mockers on users
```

---

## Notes and edge cases

- Match is substring-based, not exact path match.
- If multiple files contain the same fragment, you must provide a more specific `<NAME>`.
- Only files with standard HTTP method extensions are considered valid mock files.
- Errors while writing status are reported as `Mock is missing` or `Failed to set mock status` depending on failure stage.
