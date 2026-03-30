# `mockers info` Command Reference

## Purpose

The `info` command shows detailed metadata for a single file-based mock.

It is designed for quick inspection of:

- normalized mock path,
- response status code,
- response headers,
- delay,
- cache mode,
- detected MIME type,
- response body (optional).

---

## Syntax

```bash
mockers info [OPTIONS] <NAME>
```

### Aliases

- `show`
- `i`

---

## Parameters

### Positional

- `<NAME>` *(required)*
  - Free-form search string used to find a mock file by partial path/name.
  - Matching is **case-insensitive**.

### Options

- `--mocks <PATH>`, `-m <PATH>`
  - Path to the mocks directory.
  - If omitted, path is resolved from global config (`.mockers`) and then falls back to `./mocks`.

- `--show-body`, `-s`
  - Default: `false`.
  - Prints file contents after metadata when enabled.

---

## How matching works

`mockers info` scans the resolved mocks directory recursively and keeps only files that:

1. contain `<NAME>` in their full path (case-insensitive), and
2. have an extension that maps to a valid HTTP method (for example: `.get`, `.post`, `.put`, etc.).

Then:

- **0 matches** → prints `No mocks found`.
- **1 match** → prints full information for that mock.
- **>1 matches** → prints numbered candidates and asks for a more specific query.

> Important: the command does not use exact route matching; it uses substring matching over file paths.

---

## Output fields

For a single match, output includes:

1. **Mock** — normalized path relative to mocks root.
2. **Response status code** — from `config.json` entry or default `200`.
3. **Response headers** — from `config.json` entry (printed only when non-empty).
4. **Delay timeout (ms)** — from `config.json` entry or default `0`.
5. **Cache mode** — from `config.json` entry or default `nocache`.
6. **Detected mime** — auto-detected from body bytes.
7. **Body** — shown only with `--show-body`.

---

## Configuration interaction

## 1) Mocks directory resolution

Resolution order is:

1. `--mocks` CLI option,
2. `mocks` value from merged `.mockers` global config,
3. current working directory + `mocks`.

Notes:

- Relative paths are converted to absolute.
- `~` is expanded to the home directory.

## 2) Per-mock config lookup (`config.json`)

After selecting a mock file, `info` tries to read `config.json` in the same directory as the mock file and loads the object under key `<filename>.<method>`.

If the file is missing, unreadable, or the key is absent, defaults are used.

---

## Practical examples

### Show metadata only

```bash
mockers info users/profile.get
```

### Show metadata + body

```bash
mockers info -s users/profile
```

### Search in a custom mocks directory

```bash
mockers info -m ./fixtures api/v1/login
```

### Handle ambiguous query

```bash
mockers info user
```

If multiple files include `user`, command prints candidates and asks to refine the query.

---

## Troubleshooting

- **"No mocks found"**: check `--mocks` path and query substring.
- **Too many matches**: pass a longer, more specific path fragment.
- **Unexpected defaults**: verify that `config.json` exists in the same folder as the target mock and contains the exact filename key.
