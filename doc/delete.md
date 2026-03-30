# `delete` Command Reference

## Purpose

The `delete` command removes a single file-based mock from the mocks directory.

It performs two cleanup operations:
1. Deletes the matched mock file from disk.
2. Removes the corresponding entry from the nearest `config.json` in the same mock directory.

If `config.json` becomes empty after removal, the file itself is also deleted.

---

## Syntax

```bash
mockers delete [OPTIONS] <NAME>
```

### Aliases

The command is available through these aliases:

- `remove`
- `rm`
- `r`

---

## Parameters

### Positional

- `<NAME>` (required)
  - Free-form, case-insensitive search string.
  - Can be a full path fragment or file name fragment.
  - Matching is done against the full mock file path.

### Options

- `-m, --mocks <PATH>`
  - Path to the mock files directory.
  - If omitted, the command resolves the directory using the same precedence as other commands:
    1. `--mocks` argument
    2. `.mockers` global configuration (`mocks` field)
    3. default `./mocks` from current working directory

---

## Matching Behavior

`delete` scans mock files and keeps only candidates where:

1. Path contains `<NAME>` (case-insensitive).
2. File extension is a valid HTTP method (for example: `.get`, `.post`, `.delete`, etc.).

This prevents non-mock files from being accidentally targeted.

---

## Outcomes

### 1) No matches

The command prints:

```text
There are no mocks matching your query
```

No file changes are made.

### 2) Multiple matches

The command prints a numbered list of matched mock paths and asks for a more specific query.

No deletion happens in this case.

### 3) Exactly one match

The command removes:

- matched mock file
- matching key in the adjacent `config.json`

If all entries are removed from `config.json`, the config file is deleted.

### 4) Internal failure

The command prints:

```text
Failed to delete mock
```

---

## Configuration Notes

- `delete` depends on the global config API only for resolving the effective mocks directory.
- It does **not** use other server settings (`host`, `port`, `cors`, etc.).
- Global config file name is `.mockers` and can provide `mocks` path.

---

## Related Admin API (HTTP)

For automation or UI tooling, there is also an admin API endpoint:

- `POST /api/v2/mocks/delete`

It deletes a mock by `path + method` and updates `config.json` in a similar way.

Behavior difference vs CLI:
- API returns `404` when target file is not found.
- CLI instead relies on search results and user-facing console messages.

---

## Practical Tips

- Prefer specific `<NAME>` values (full path fragment + method filename) to avoid ambiguous matches.
- Use `mockers list` first when unsure about exact file names.
- In scripts, prefer admin API if you need explicit HTTP status codes for control flow.
