# `mockers disable` Command Reference

## Purpose

The `disable` command marks a specific file-based mock as disabled in its local `config.json` entry.

When a mock is disabled:
- it is ignored during file-based response lookup;
- Mockers falls back to proxy behavior (if `--origin` is configured) or returns `404`.

---

## Command Syntax

```bash
mockers disable [OPTIONS] <NAME>
```

Also documented with `enable` as a pair of toggling commands.

---

## Parameters

### Positional

- `<NAME>` (required)
  - Partial or full mock file name/path.
  - Matching is **case-insensitive**.
  - Example values: `users.get`, `foo/bar/users.get`, `users`.

### Options

- `-m, --mocks <PATH>`
  - Path to the directory containing mock files.
  - If omitted, Mockers resolves the mocks path from global configuration.

---

## How Matching Works

The command scans files under the resolved mocks directory and filters candidates by:

1. file path contains `<NAME>` (case-insensitive),
2. file extension is a valid HTTP method (`.get`, `.post`, etc.).

Then:

- **0 matches** → prints: `There are no mocks matching your query`.
- **>1 matches** → prints numbered candidates and asks for a more specific query.
- **exactly 1 match** → updates that mock entry in `config.json`.

---

## Configuration Behavior (`config.json`)

For the selected mock file, Mockers writes/updates an entry in the sibling `config.json`:

- entry key = file name only (e.g., `users.get`),
- field `disabled` is set to `true`.

If `config.json` does not exist, it is created.
If the entry does not exist, it is created with default config values plus `disabled: true`.

### Effective schema impact

`disable` affects one config field:

- `disabled: true`

Other settings (`statusCode`, `delayMs`, `headers`, `cacheMode`) remain as existing values when present, or defaults for a new entry.

---

## Runtime Effect During `serve`

At request handling time, Mockers:

1. resolves requested mock file path (`<route>.<method>`),
2. reads matching config entry,
3. checks `is_disabled()`.

If disabled:
- file content is **not** served;
- request goes to fallback flow:
  - proxy to `--origin` if configured,
  - otherwise `404 Not Found`.

---

## Related Command

- `mockers enable [OPTIONS] <NAME>`
  - sets `disabled: false` for the same selection logic.

---

## Examples

```bash
# Disable by exact file name
mockers disable users.get

# Disable by partial path
mockers disable api/v1/users

# Disable in a specific mocks directory
mockers disable --mocks ./mocks users
```

---

## Notes and Caveats

- Selection is substring-based, not exact path matching.
- If multiple files match, nothing is changed until query is refined.
- `disable` does not delete mock files.
- The command updates configuration only for one resolved mock.
