# `serve` Command in Mockers

## Purpose

`mockers serve` starts the HTTP mock server runtime. It:

- serves responses from file-based mocks (`<path>.<method>`),
- applies per-mock settings from nearby `config.json`,
- forwards to an upstream (`--origin`) when a matching mock is missing,
- can persist upstream responses back to disk when `cacheMode=overwrite`.

This is the primary command used to run Mockers as a server process.

---

## Syntax

```bash
mockers serve [OPTIONS]
```

Command aliases: `run`, `start`, `s`.

---

## Command options

### Network and paths

- `--host <HOST>` — interface/host to bind.
  - Default: `0.0.0.0`.
- `--port, -p <PORT>` — server port.
  - Default: `8080`.
- `--mocks, -m <MOCKS>` — path to the mocks directory.
  - Default: `mocks` (relative to current working directory).
  - If the directory does not exist, it is created during parameter validation.

### CORS and preflight

- `--cors, -c[=<true|false>]` — CORS toggle.
  - Can be passed as `--cors` (equivalent to `true`) or explicitly as `--cors=false`.
  - Enables CORS headers on server responses.
- `--preflight <mirror|permissive>` — automatic handling of browser preflight (`OPTIONS`) requests.
  - `mirror` — mirrors requested origin/method/headers.
  - `permissive` — broad allow-all style behavior.

### Response behavior

- `--delay-ms, -d <DELAY_MS>` — global response delay in milliseconds.
  - Used as fallback when `delayMs` is not set for a specific mock in `config.json`.
- `--origin, -o <ORIGIN>` — upstream URL used when a file-based mock is missing (or disabled).
- `--proxy-body-max-bytes <N>` — max allowed upstream response body size.
  - Default: `16 MiB`.
  - Hard cap: `32 MiB` (values above this are clamped).

### Upstream proxy

- `--proxy <PROXY>` — proxy connection string for upstream requests.
  - Supports proxies compatible with `reqwest`: `socks5h://127.0.0.1:1080`, `http://proxy-gateway:8080`, `https://proxy-gateway:443`.
  - Used only when proxying to `--origin`.
  - If unset, upstream requests go through direct connection (`no_proxy`).

### Admin and logging

- `--admin-base-url, -a <PATH>` — enables admin API + Swagger under the given absolute base path.
  - Must be an absolute path (for example, `/__admin`).
  - Relative values fail validation.
- `--log-request, -l <info|debug|trace>` — inbound request logging detail:
  - `info`: method + URI,
  - `debug`: method + URI + headers,
  - `trace`: method + URI + headers + body.
- `--verbosity, -v <info|debug|trace>` — process/service verbosity.
  - With `debug`/`trace`, startup parameters are additionally printed.

---

## Configuration precedence

For most `serve` parameters (`host`, `port`, `mocks`, `cors`, `preflight`, `delay`, `origin`,
`admin_base_url`, `log_request`, `verbosity`, `proxy_body_max_bytes`, `proxy`), the resolution order is:

1. CLI flag value,
2. global `.mockers` configuration value,
3. built-in default.

Global config itself may be merged from multiple `.mockers` files across the directory hierarchy
(root to current directory), where more local files override more global ones.

---

## `serve` lifecycle

1. Parse CLI (`clap`) and select `Serve` subcommand.
2. Validate params via `test()`:
   - ensure/create mocks directory,
   - validate `admin_base_url`.
3. Create TCP listener on `host:port`.
4. Build router pipeline:
   - pre-middleware request logger,
   - pre-middleware HTTP method validator,
   - primary `mock_handler`,
   - error handler.
5. If `admin_base_url` is set, mount admin API/Swagger scope.
6. Enter accept loop with graceful shutdown signal handling.

---

## Runtime request handling flow

For each request:

1. Build mock file name: `"<path>.<method_lowercase>"`.
2. Read local `config.json` for the current mock directory (if present).
3. If request is preflight and `--preflight` is enabled, return `204` with preflight/CORS headers.
4. If file mock exists and is not disabled, respond from disk with:
   - `statusCode`,
   - `headers`,
   - `delayMs` (or global delay),
   - MIME inferred from body content.
5. If mock is missing/disabled:
   - without `--origin` → return `404`,
   - with `--origin` → proxy request upstream.
6. Enforce `proxy_body_max_bytes` for upstream response body; overflow/errors return `502`.
7. If `cacheMode=overwrite`, asynchronously persist upstream response + metadata as mock files.

---

## Important behavior and edge cases

- If `--mocks` points to an existing file/symlink instead of a directory, startup fails.
- If `--origin` is invalid (target URL cannot be built), request returns `400`.
- Upstream transport failures return `502 Bad Gateway`.
- Oversized `--proxy-body-max-bytes` is clamped to hard maximum.
- `--cors` and `--preflight` are independent toggles (commonly used together).

---

## Examples

### Basic run

```bash
mockers serve
```

### With CORS and preflight

```bash
mockers serve --cors --preflight=permissive
```

### Mock-first with upstream fallback

```bash
mockers serve -m ./mocks -o http://localhost:9000 --proxy-body-max-bytes 4194304
```

### Enable admin API

```bash
mockers serve --admin-base-url=/__admin
```

### HTTPS support

`mockers serve` can run with HTTPS enabled.

To turn it on, put a TLS certificate and private key into your `mocks` directory and name them `cert.pem` and `key.pem`. No magic, just files.

Mockers uses a separate port for HTTPS, so HTTP and HTTPS do not have to fight over the same one. Configure it with the `httpsPort` field in the global config file or with the `--https-port` flag.

If `cert.pem`, `key.pem`, and an HTTPS port are present, Mockers will start serving HTTPS.

For local development, you can generate certificates with any tool you prefer. Mockers does not care. If you want the easy route, use [mkcert](https://github.com/filosottile/mkcert). It can create a local root CA, add it to your system trust store, and issue a certificate for `127.0.0.1` or a local domain.
