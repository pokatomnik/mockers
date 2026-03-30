# `mockers list` Command Reference

## Purpose

The `list` command scans the mocks directory and prints all detected file-based mocks.

A file is treated as a mock only if:

1. It is a regular file.
2. Its extension is a valid HTTP method (for example: `.get`, `.post`, `.put`).
3. The HTTP method belongs to the standard method set supported by Mockers (`GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `OPTIONS`, `HEAD`, `TRACE`, `CONNECT`).

This command is read-only: it does not create, modify, or delete files.

---

## Syntax

```bash
mockers list [OPTIONS]
```

Aliases:

- `mockers ls`

---

## Options

### `-m, --mocks <MOCKS>`

Path to the directory containing mock files.

Accepted forms:

- absolute path (`/opt/mocks`)
- relative path (`./mocks`, `../fixtures/mocks`)
- home-relative path (`~/mocks`)

If the path is relative, Mockers resolves it to an absolute path using the current working directory.

---

## Mocks path resolution (configuration behavior)

When deciding which directory to scan, Mockers uses this precedence:

1. `--mocks` CLI argument (highest priority)
2. `mocks` value from merged global `.mockers` config
3. default `<current_working_directory>/mocks`

Global config is merged from `.mockers` files found along the current directory ancestry chain.

---

## How detection works

`list` recursively walks the target directory tree and evaluates each file:

- Directories are traversed recursively.
- Non-file entries are ignored.
- For each file, the final extension is parsed as HTTP method (case-insensitive).
- Only files with standard HTTP methods are included.

Examples:

- `users/list.get` -> included as `GET /users/list`
- `auth/login.POST` -> included as `POST /auth/login`
- `health.custom` -> ignored (invalid method)
- `orders.get.json` -> ignored (`json` is treated as extension, not `get`)

---

## Output format

### No mocks found

```text
There are no mocks
```

### Mocks found

First line:

```text
There are some mocks in <absolute_path>
```

Then one line per mock:

```text
* <METHOD> <pathname_without_extension>
```

Example:

```text
There are some mocks in /workspace/project/mocks
* GET /users/list
* POST /auth/login
```

---

## Operational notes

- The command can return an error if Mockers fails to determine any mocks path (for example, unusual environment failures resolving directories).
- Method names are normalized to uppercase in output.
- The command does not validate JSON content of mock bodies.
- The command does not require `config.json`; it lists files by naming convention.

---

## Quick examples

```bash
# Use default path: <cwd>/mocks (or .mockers value if configured)
mockers list

# Explicit directory
mockers list --mocks ./test/mocks

# Alias
mockers ls -m ~/sandbox/mocks
```
