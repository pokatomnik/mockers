# `mockers config` Command Reference

## Purpose

The `config` command prints the **effective global Mockers configuration** resolved for the current working directory.

It is intended for:

- verifying what values Mockers will use when command-line flags are not passed,
- troubleshooting unexpected runtime behavior,
- validating hierarchical `.mockers` file resolution.

---

## Command and aliases

```bash
mockers config
```

Supported aliases:

```bash
mockers configuration
mockers settings
mockers preferences
mockers prefs
```

---

## Parameters

The `config` command has **no positional parameters and no options**.

It only reads configuration from `.mockers` files and prints resolved values.

---

## Where configuration comes from

Mockers uses a global configuration file named:

```text
.mockers
```

When `mockers config` runs, Mockers:

1. takes the current working directory,
2. builds a directory chain up to filesystem root,
3. checks each directory for `.mockers`,
4. deserializes valid JSON config files,
5. merges them into one resulting config.

### Merge behavior (important)

The closest `.mockers` file to your current directory has higher priority than parent directories.

In practice:

- parent directory config provides base defaults,
- nested project config overrides only specified fields,
- unspecified fields are inherited.

---

## Printed fields

`mockers config` prints a human-readable report with these fields:

- `Host`
- `Port`
- `HTTPS Port`
- `Mocks path`
- `Cors`
- `Preflight`
- `Delay in milliseconds`
- `Origin`
- `Admin base URL`
- `Requests log level`
- `Verbosity level`
- `Proxy body max bytes`
- `Proxy` (shows `Proxy IS set` or `Proxy is NOT set`)

If a value is not set in any discovered `.mockers`, it is displayed as:

```text
[unset]
```

---

## Configuration model (JSON keys)

The global configuration structure supports these keys (`camelCase`):

- `host` (string)
- `port` (number)
- `httpPort` (number)
- `mocks` (string)
- `cors` (boolean)
- `preflight` (`"mirror"` or `"permissive"`)
- `delayMs` (number)
- `origin` (string)
- `adminBaseUrl` (string)
- `logRequest` (`"info"`, `"debug"`, `"trace"`)
- `verbosity` (`"info"`, `"debug"`, `"trace"`)
- `proxyBodyMaxBytes` (number)
- `proxy` (string) — proxy connection string for upstream requests (socks5, http, https)

---

## Relation to `init`

You can create a user-level `~/.mockers` using:

```bash
mockers init
```

or interactive mode:

```bash
mockers init --interactive
```

`init` writes a default global config (host, port, https port, mocks, CORS, preflight, delay, admin base URL, log level, verbosity).

---

## Effective defaults at runtime vs. `[unset]` output

`mockers config` reflects only values found in `.mockers` files.

For `serve`, additional built-in runtime defaults are applied when values are missing:

- `host = 0.0.0.0`
- `port = 8080` (or `8443` when HTTPS enabled)
- `mocks = mocks`
- `cors = false`
- `delayMs = 0`
- `logRequest = info`
- `verbosity = info`
- `proxyBodyMaxBytes = 16777216` (hard capped at `33554432`)

So seeing `[unset]` in `mockers config` does **not** necessarily mean the server has no final value; some values are filled by runtime defaults.

---

## Example

```text
Global config:
=================
Host:            0.0.0.0
Port:            8080
HTTPS Port:      8443
Mocks path:      mocks
Cors:            false
Preflight:       permissive
Delay in milliseconds: 0
Origin:          [unset]
Admin base URL:  /__admin
Requests log level: info
Verbosity level: info
```

---

## Troubleshooting tips

- Run `mockers config` from the same directory where you run `mockers serve`.
- If output is unexpected, check for additional `.mockers` in parent folders.
- Validate JSON syntax in every `.mockers`; invalid files are ignored during deserialization.
- If a field is `[unset]`, either add it to `.mockers` or provide the corresponding CLI flag for `serve`.
