# `mockers create` Command Reference

## Purpose
The `create` command generates a new file-based mock and ensures its runtime configuration is present.

In one run, it:
1. Resolves the target mocks directory.
2. Creates missing directories on disk.
3. Creates/overwrites the mock response file (`<name>.<method>`).
4. Creates or updates `config.json` in the same directory with metadata for that mock.

---

## Syntax

```bash
mockers create [OPTIONS] <ROUTE>
```

Aliases for the subcommand:
- `new`
- `n`

---

## Parameters

### Positional
- `ROUTE` (required): URL-like route used to build the file path.
  - Examples: `/users/profile`, `users/profile`

### Options

| Option | Type | Default | Notes |
|---|---|---|---|
| `--method` | string | `GET` | HTTP method for mock filename extension, lowercased in filename. |
| `--status-code`, `-s` | `u16` | `200` | Stored in `config.json`. |
| `--delay-ms`, `-d` | `u64` | `0` | Stored in `config.json`; optional in CLI, but written with default if omitted. |
| `--header` | repeated string | none | Format: `Key: Value`. Can be repeated multiple times. |
| `--contents`, `-c` | string | `{"hello":"world"}` | Raw file body content written to mock file. |
| `--cache-mode` | enum | `nocache` | Allowed: `overwrite`, `nocache`. |
| `--mocks`, `-m` | string path | from global config or `./mocks` | Target root directory for mock files. |
| `--disabled` | bool flag | `false` | Stores disabled state in `config.json`. |

---

## Output Structure

For `mockers create /users/profile --method GET` with default `--mocks`:

```text
mocks/
└── users/
    └── profile.get
    └── config.json
```

`config.json` contains an entry keyed by mock filename:

```json
{
  "profile.get": {
    "delayMs": 0,
    "statusCode": 200,
    "headers": {},
    "cacheMode": "nocache",
    "disabled": false
  }
}
```

---

## How Path Resolution Works

`--mocks` resolution priority:
1. CLI argument `--mocks`.
2. Global `.mockers` configuration (`mocks` field).
3. Fallback to `<current_working_directory>/mocks`.

Path behavior:
- Relative paths are converted to absolute paths using current working directory.
- `~` prefix is expanded to home directory.
- If target does not exist, directories are created recursively.
- If target exists but is not a directory (file/symlink), command fails.

---

## Header Parsing Behavior

Each `--header` value is parsed via first `:` character:
- Valid: `"X-Env: local"` → key `X-Env`, value `local`
- Invalid or empty key/value entries are silently ignored.

Implication: malformed headers do not stop the command; they are just not written into config.

---

## Write/Overwrite Semantics

- Mock response file is always written (overwritten if exists).
- `config.json` behavior:
  - If exists and valid JSON map, target entry is inserted/updated.
  - If missing or unreadable, a new map is created with one entry.
- File body is written concurrently with config update.

---

## Error Cases

Typical failures:
- Cannot resolve or create mocks directory.
- Target `--mocks` points to non-directory.
- Route cannot produce a valid filename component.
- Filesystem write errors (permission, disk, etc.).

Errors are bubbled to top-level command handler and printed as logged failures.

---

## Practical Examples

### Minimal
```bash
mockers create /users/profile
```

### With custom metadata
```bash
mockers create /users/profile \
  --method POST \
  --status-code 201 \
  --delay-ms 250 \
  --header "Content-Type: application/json" \
  --header "X-Env: local" \
  --contents '{"id":1,"name":"Alice"}' \
  --cache-mode overwrite
```

### Create disabled mock in custom directory
```bash
mockers create /billing/invoice \
  --mocks ./fixtures/mocks \
  --disabled
```

---

## Configuration Impact Summary

`create` affects two storage layers:
1. **Mock body file** for payload.
2. **`config.json`** for runtime behavior:
   - status code
   - delay
   - headers
   - cache mode
   - disabled flag

This split allows changing behavior metadata independently from response body content.
