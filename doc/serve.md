# `serve` Command in Mockers

## Purpose

`mockers serve` starts the HTTP mock server runtime. It:

- serves responses from file-based mocks (`<path>.<method>`),
- can generate mock responses from LLM prompt files with YAML frontmatter,
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
3. If the mock is disabled in `config.json`, return `404` immediately.
4. If request is preflight and `--preflight` is enabled, return `204` with preflight/CORS headers.
5. If file mock exists, is UTF-8 text, and starts with `$mockers.prompt: true` frontmatter, call the configured LLM provider and return generated content.
6. Otherwise, if file mock exists, respond from disk with:
   - `statusCode`,
   - `headers`,
   - `delayMs` (or global delay),
   - MIME inferred from body content.
7. If mock is missing:
   - without `--origin` → return `404`,
   - with `--origin` → proxy request upstream.
8. Enforce `proxy_body_max_bytes` for upstream response body; overflow/errors return `502`.
9. If `cacheMode=overwrite`, asynchronously persist upstream response + metadata as mock files.

---

## LLM prompt mocks and frontmatter

Mockers can treat a mock file as a prompt and generate the response body through an OpenAI-compatible provider.
This happens only for existing UTF-8 mock files with valid YAML frontmatter and `$mockers.prompt: true`.

Example `mocks/users/profile.get`:

```md
---
$mockers:
  prompt: true
  api_endpoint: https://api.openai.com/v1/chat/completions
  env_key: OPENAI_API_KEY
  model: gpt-4o-mini
  proxy: http://localhost:3128
  ttl: 60000
---
Return a realistic JSON profile for the requested user.
The response must be a JSON object with id, name, email, and role.
```

### Frontmatter rules

- Frontmatter must be the first content in the file.
- The opening and closing delimiters must be lines containing exactly `---`.
- Frontmatter is YAML.
- Mockers reads only the literal `$mockers` key for prompt configuration.
- `$mockers.prompt: true` enables LLM handling.
- If `$mockers.prompt` is missing, `false`, invalid, or the frontmatter cannot be parsed, Mockers serves the file as a normal static mock.

### `$mockers` fields

| Field          | Type    | Required | Description                                                                 |
| -------------- | ------- | -------- | --------------------------------------------------------------------------- |
| `prompt`       | boolean | yes      | Enables LLM generation when `true`                                           |
| `api_endpoint` | string  | yes      | OpenAI-compatible chat completions endpoint                                  |
| `model`        | string  | yes      | Model name sent to the provider                                              |
| `env_key`      | string  | no       | Environment variable containing a bearer token for `Authorization`           |
| `proxy`        | string  | no       | Proxy URL for the LLM request only; independent from `--proxy`/`--origin`    |
| `ttl`          | number  | no       | In-memory generated-response cache TTL in milliseconds; default is `0`       |

### Provider request and response behavior

Mockers sends an OpenAI-style chat completion request:

- `model` is taken from frontmatter,
- `messages` contains a system message and a user message,
- the user message is the mock file content,
- the system message includes Mockers generation rules plus current request details: method, URI, headers, and body,
- `stream` is always `false`,
- `temperature` is always `0`.

Mockers expects an OpenAI-like response with `choices[0].message.content`. The first choice must have `finish_reason: "stop"` and an assistant message. That content becomes the HTTP response body, with MIME type inferred from generated bytes.

Per-mock `statusCode` and `headers` from `config.json` are applied to generated responses. If the LLM call fails, returns a non-success status, required fields are missing, or the provider response shape is unsupported, Mockers returns `502 Bad Gateway`.

---

## Important behavior and edge cases

- If `--mocks` points to an existing file/symlink instead of a directory, startup fails.
- A mock with `disabled: true` returns `404` immediately and does not fall back to `--origin`.
- If `--origin` is invalid (target URL cannot be built), request returns `400`.
- Upstream transport failures return `502 Bad Gateway`.
- LLM provider failures also return `502 Bad Gateway`.
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
