# Mockers — tiny but powerful HTTP mock server in Rust 🎯

![Mockers](./.assets/mockers-logo.svg)
[![Rust](https://github.com/pokatomnik/mockers/actions/workflows/rust.yml/badge.svg)](https://github.com/pokatomnik/mockers/actions/workflows/rust.yml)

`Mockers` is a CLI + HTTP server that lets you spin up fake APIs from files in seconds.
If you need a quick backend for frontend work, contract testing, demos, QA, or local integration tests — this thing is exactly for that. No heavy setup, no DB, no headache.

---

## Why this project exists

You have requests coming in.
You want deterministic responses.
You want them fast.

`Mockers` maps URL + HTTP method to files, returns file contents as responses, and gives you extra controls like:

- custom status codes,
- response delay,
- custom headers,
- admin API,
- request forwarding to a real origin,
- optional disk write-through caching.

So it can work as both:

- a pure mock server, and
- a "mock-first, proxy-if-missing" server.

---

## Installation

Build from source:

```bash
cargo build --release
```

Binary will be in `target/release/mockers`.

---

## Quick start

Run server with defaults:

```bash
mockers serve
```

Defaults:

- host: `0.0.0.0`
- port: `8080`
- https port: `8443`
- mocks dir: `./mocks`

If `./mocks` does not exist, Mockers creates it.

---

## CLI commands overview

```bash
mockers <COMMAND>
```

Available commands:

- `serve` — run HTTP server
- `config` — show effective global Mockers configuration
- `init` — initialize global Mockers configuration file
- `completion` — generate shell completion script

Aliases exist for `serve`, `config`, and `init`; check `--help`.

---

## Global configuration file (`.mockers`)

Mockers supports a **hierarchical global configuration** system via `.mockers` files.

When any command runs, Mockers walks from the current working directory up to the filesystem root,
reads every `.mockers` file it finds, and merges them. Config fields from closer directories override
those from parent directories.

You can create a user-level global config in your home directory:

```bash
mockers init           # non-interactive, writes defaults
mockers init -i        # interactive setup with prompts
```

The file is written to `$HOME/.mockers` in JSON format.

CLI flags always take precedence over global config values, which in turn override built-in defaults.

Supported `.mockers` fields (all optional, `camelCase` keys):

| Field               | Type                               | Description                                                         |
| ------------------- | ---------------------------------- | ------------------------------------------------------------------- |
| `host`              | string                             | Interface to bind                                                   |
| `port`              | number                             | HTTP port                                                           |
| `httpsPort`         | number                             | HTTPS port                                                          |
| `mocks`             | string                             | Path to mocks directory                                             |
| `cors`              | boolean                            | Enable CORS headers                                                 |
| `preflight`         | `"mirror"` \| `"permissive"`       | Preflight handling                                                  |
| `delayMs`           | number                             | Global response delay (ms)                                          |
| `origin`            | string                             | Fallback upstream URL                                               |
| `adminBaseUrl`      | string                             | Admin API base path                                                 |
| `logRequest`        | `"info"` \| `"debug"` \| `"trace"` | Request logging level                                               |
| `verbosity`         | `"info"` \| `"debug"` \| `"trace"` | Process verbosity level                                             |
| `proxyBodyMaxBytes` | number                             | Max upstream response body (bytes)                                  |
| `proxy`             | string                             | Proxy connection string for upstream requests (socks5, http, https) |

---

## `serve` command

```bash
mockers serve [OPTIONS]
```

### Options

| Flag                     | Default             | Description                                                                                                     |
| ------------------------ | ------------------- | --------------------------------------------------------------------------------------------------------------- |
| `--host`                 | `0.0.0.0`           | Interface to bind to                                                                                            |
| `--port`, `-p`           | `8080`              | Port to listen on                                                                                               |
| `--https-port`           | `8443`              | HTTPS port to listen on                                                                                         |
| `--mocks`, `-m`          | `mocks`             | Directory with mock files                                                                                       |
| `--cors`, `-c`           | `false`             | Adds CORS headers (`Access-Control-Allow-Origin: *`). Can be passed as `--cors` (true) or `--cors=false`        |
| `--preflight`            | unset               | Auto-handle browser OPTIONS preflight requests                                                                  |
| `--delay-ms`, `-d`       | `0`                 | Global response delay in ms                                                                                     |
| `--origin`, `-o`         | unset               | Fallback upstream server when mock is missing                                                                   |
| `--admin-base-url`, `-a` | unset               | Enables admin API + Swagger under given absolute base path                                                      |
| `--log-request`, `-l`    | `info`              | Request logging level: `info`, `debug`, `trace`                                                                 |
| `--verbosity`, `-v`      | `info`              | Process/service verbosity level: `info`, `debug`, `trace`. With `debug`/`trace`, startup parameters are printed |
| `--proxy-body-max-bytes` | `16777216` (16 MiB) | Maximum upstream response body size when proxying. Hard capped at 32 MiB                                        |
| `--proxy`                | unset               | Proxy connection string for upstream requests. Supports socks5, http, https — e.g. `socks5h://127.0.0.1:1080`   |

### Graceful shutdown

Mockers handles `Ctrl+C` (SIGINT) for a clean server shutdown. When interrupted, it stops accepting new connections and lets in-flight requests complete.

### HTTPS support

`mockers serve` can run with HTTPS enabled.

To turn it on, put a TLS certificate and private key into your `mocks` directory and name them `cert.pem` and `key.pem`. No magic, just files.

Mockers uses a separate port for HTTPS, so HTTP and HTTPS do not have to fight over the same one. Configure it with the `httpsPort` field in the global config file or with the `--https-port` flag.

If `cert.pem`, `key.pem`, and an HTTPS port are present, Mockers will start serving HTTPS.

For local development, you can generate certificates with any tool you prefer. Mockers does not care. If you want the easy route, use [mkcert](https://github.com/filosottile/mkcert). It can create a local root CA, add it to your system trust store, and issue a certificate for `127.0.0.1` or a local domain.

### Preflight modes

If `--preflight` is enabled, Mockers can answer browser preflight requests automatically:

- `mirror` — mirrors requested origin/method/headers back to the browser.
- `permissive` — basically "allow everything" mode (`*`, all methods, etc.).

Good for local dev when CORS fights you.

### Request logging levels

- `info`: method + path/query.
- `debug`: method + path/query + headers.
- `trace`: method + path/query + headers + body.

### Verbosity levels

Controls process/service output (separate from request logging):

- `info`: normal startup/shutdown messages.
- `debug`: startup parameters and additional diagnostics.
- `trace`: detailed internal tracing.

---

## How file-based routing works

Mock file naming convention is:

```text
<route_path>.<http_method_lowercase>
```

Examples:

```text
user.get       -> GET /user
login.post     -> POST /login
config.put     -> PUT /config
```

Nested routes are just nested folders/files:

```text
mocks/
├─ users/
│  ├─ list.get
│  └─ create.post
└─ auth/
   └─ login.post
```

This gives you:

- `GET /users/list`
- `POST /users/create`
- `POST /auth/login`

### Reading a real `mocks/` tree

When your project already has a populated `mocks/` directory, you can read it as a contract map.

Example:

```text
mocks/
├─ auth/
│  ├─ login.post
│  ├─ refresh.post
│  └─ config.json
├─ users/
│  ├─ me.get
│  ├─ me.patch
│  └─ config.json
└─ health.get
```

How to interpret this:

- `auth/login.post` means `POST /auth/login`
- `auth/refresh.post` means `POST /auth/refresh`
- `users/me.get` means `GET /users/me`
- `users/me.patch` means `PATCH /users/me`
- `health.get` means `GET /health`

Important details:

- File extension is always the HTTP method (`.get`, `.post`, `.patch`, ...).
- Path segments come from folders + filename stem.
- `config.json` is local to its folder and config keys must match mock filenames in that same folder.
- Body is returned exactly from file content, while `Content-Type` is auto-detected from bytes (JSON/CSS/text/binary, etc.).

---

## LLM prompt mocks with frontmatter

A mock file can be treated as a prompt for an OpenAI-compatible LLM provider instead of being returned as static bytes.
To enable this, the file must be UTF-8 text and start with YAML frontmatter delimited by exact `---` lines.

Example `mocks/users/profile.get`:

```md
---
$mockers:
  prompt: true
  api_endpoint: https://api.openai.com/v1/chat/completions
  env_key: OPENAI_API_KEY
  model: gpt-4o-mini
  proxy: socks5h://127.0.0.1:1080
  ttl: 60000
---

Return a realistic JSON user profile for the current request.
Use the request path and headers when useful.
```

Frontmatter format:

- Frontmatter must be at the very beginning of the mock file.
- Opening and closing delimiters must be lines containing exactly `---`.
- The Mockers-specific config lives under the literal `$mockers` YAML key.
- `$mockers.prompt: true` is required to call the LLM. If it is missing or `false`, the file is served as a normal static mock.
- Invalid or unparsable frontmatter is ignored and the file is served as a normal static mock.

Supported `$mockers` fields:

| Field          | Type    | Required | What it does                                                           |
| -------------- | ------- | -------- | ---------------------------------------------------------------------- |
| `prompt`       | boolean | yes      | Enables LLM generation when `true`                                     |
| `api_endpoint` | string  | yes      | OpenAI-compatible chat completions endpoint                            |
| `model`        | string  | yes      | Model name sent in the provider request                                |
| `env_key`      | string  | no       | Environment variable containing a bearer token for `Authorization`     |
| `proxy`        | string  | no       | Proxy URL used only for this LLM provider request                      |
| `ttl`          | number  | no       | In-memory generated-response cache TTL in milliseconds; default is `0` |

When a prompt mock is handled, Mockers sends the mock file content as the user prompt and appends request context to the system prompt: method, URI, headers, and body. The provider request uses `stream: false` and `temperature: 0` for deterministic responses.

The provider response must look like an OpenAI chat completion response. Mockers uses the first choice only, requires `finish_reason: "stop"`, requires assistant role, and returns `message.content` as the HTTP response body. `Content-Type` is inferred from the generated body. Per-mock `statusCode` and `headers` from `config.json` are still applied.

If the LLM request fails, the provider returns a non-success status, required frontmatter fields are missing, or the response shape is unsupported, Mockers returns `502 Bad Gateway` for that request.

---

## Mock config file (`config.json`)

In any mock directory, you can add a `config.json` file.
Keys are mock file names in the same directory (like `user.get`).

Example:

```json
{
  "user.get": {
    "delayMs": 1200,
    "statusCode": 201,
    "headers": {
      "X-Server": "Mockers"
    },
    "cacheMode": "overwrite",
    "disabled": false
  }
}
```

### Supported fields

| Field        | Type                     | What it does                                              |
| ------------ | ------------------------ | --------------------------------------------------------- |
| `delayMs`    | number                   | Per-mock delay in ms                                      |
| `statusCode` | number                   | Per-mock HTTP status                                      |
| `headers`    | object string->string    | Extra response headers                                    |
| `cacheMode`  | `overwrite` \| `nocache` | Controls write-through behavior when proxying to `origin` |
| `disabled`   | boolean                  | If `true`, mock returns `404` immediately                 |

### Precedence and defaults

For a matching mock file:

- status defaults to `200`
- delay defaults to global `--delay-ms` value (or `0`)
- headers default to empty
- cache mode defaults to `nocache`
- disabled defaults to `false`

If a config key is missing, defaults are used.

---

## Request handling flow (important)

For every incoming request, Mockers roughly does this:

1. Build mock filename from request path + method.
2. Read adjacent `config.json`.
3. If the mock is disabled, return `404` immediately.
4. Handle preflight requests when `--preflight` is enabled.
5. If the mock file exists and has `$mockers.prompt: true` frontmatter, try to generate the response through the configured LLM provider.
6. Otherwise, try the static file-based mock.
7. If file mock is missing:
   - if `--origin` is set -> proxy request to origin,
   - else -> return `404`.
8. If proxied and `cacheMode=overwrite`, write response to disk asynchronously (`mock file + config.json`).

So you can warm up mocks from a real backend automatically.

---

## `config` command

```bash
mockers config
```

Shows the **effective global Mockers configuration** resolved for the current directory.
Values are pulled from `.mockers` files (from CWD up to root) and merged.

Aliases: `configuration`, `settings`, `preferences`, `prefs`.

Takes no parameters. It prints a human-readable report with all supported fields:
host, port, HTTPS port, mocks path, CORS, preflight, delay, origin, admin base URL,
request log level, verbosity level, proxy body max bytes, proxy.

If a value is not set in any `.mockers` file, it shows as `[unset]` (except `proxy`,
which displays as `Proxy IS set` or `Proxy is NOT set`).

---

## `init` command

```bash
mockers init [OPTIONS]
```

Initializes the global Mockers configuration file at `$HOME/.mockers`.

Alias: `setup`.

### Options

| Flag                  | Default | Description                                         |
| --------------------- | ------- | --------------------------------------------------- |
| `--interactive`, `-i` | `false` | Interactive mode with prompts for each config field |

Without `--interactive`, writes built-in defaults to `~/.mockers`.

If `~/.mockers` already exists, you will be asked for confirmation before overwriting.

---

## Admin API and Swagger UI

Enable admin surface with:

```bash
mockers serve --admin-base-url /__admin
```

Then you'll get:

- REST API under `/__admin/api/v2/...`
- Swagger UI under `/__admin/swagger`

The admin API works directly with file-based mocks on disk.
Creating a mock creates the mock file and updates `config.json`.
Deleting a mock removes the mock file and cleans up its config entry.
Listing mocks returns only route path + HTTP method pairs.

### Admin API routes

- `POST /api/v2/mocks` — get all mocks (`path` + `method` only)
- `POST /api/v2/mocks/create` — create a new file-based mock
- `POST /api/v2/mocks/config` — get mock config by path+method
- `POST /api/v2/mocks/delete` — delete a file-based mock by path+method

Swagger also documents request/response schemas and known admin error codes.

---

## CORS behavior

Two separate things exist:

1. `--cors` adds regular `Access-Control-Allow-Origin: *` to regular responses.
2. `--preflight` handles OPTIONS preflight negotiation automatically.

Use both if you want easiest browser interop in local env.

---

## Typical workflows

### 1) Pure local mocks

```bash
mockers serve --mocks ./mocks
```

### 2) Mock + fallback to real backend

```bash
mockers serve --mocks ./mocks --origin https://example-api.dev
```

Optional: set `cacheMode: "overwrite"` for selected mocks to save upstream responses to disk.

### 3) Frontend local dev with CORS painkillers

```bash
mockers serve --cors --preflight permissive
```

### 4) File mock management via admin API

```bash
mockers serve --admin-base-url /__admin
```

Open: `http://localhost:8080/__admin/swagger`

---

## Notes and caveats

- Admin base URL must be an absolute path (like `/__admin`).
- Mockers only treats files with valid HTTP method extensions as mocks.
- If a path in `--mocks` points to a file/symlink instead of dir, command fails.
- Relative `--mocks` paths are resolved from current working directory.
- Response body mime is auto-detected from content.

---

## Config schema

JSON schema for `config.json`:

- [`schemas/config.v1.json`](./schemas/config.v1.json)

---

## Docker

### Build the image

```sh
docker build -t mockers-dockers .
```

This builds a production image for mockers.

### Run the container

The mocks directory must be mounted from the host into /app/mocks inside the container.

### Basic example

```sh
docker run --rm \
  -p 8080:8080 \ # if mockers serves HTTP
  -p 8443:8443 \ # if mockers serves HTTPS
  -v "/path/to/host/mocks/directory:/app/mocks" \
  mockers-dockers serve
```

This will:

- expose the server on port `8080` (or `8443` when HTTPS enabled, see HTTPS support)
- mount the local `/path/to/host/mocks/directory` directory into the container as `/app/mocks`
- start `mockers serve`

### Port mapping

The application always listens on port `8080` or `8443` inside the container.

To expose it on a different host port, change the Docker port mapping:

```sh
docker run --rm \
  -p 9090:8080 \
  -v "/path/to/host/mocks/directory:/app/mocks" \
  mockers-dockers serve
```

In this example:

- host port: `9090`
- container port: `8080`

### Mocks directory

The container always expects mocks at:

```sh
/app/mocks
```

A local directory must be mounted there when the container is started.

Read-only mount is recommended if mockers only needs to read mock files:

```sh
-v "/path/to/host/mocks/directory:/app/mocks:ro"
```

If write access is required, remove `:ro`:

```sh
-v "/path/to/host/mocks/directory:/app/mocks"
```

### Full example

```sh
docker build -t mockers-dockers .

docker run --rm \
  -p 8080:8080 \
  -v "/path/to/host/mocks/directory:/app/mocks:ro"
  mockers-dockers serve \
  --cors \
  --delay-ms 150 \
  --log-request debug
```

---

## Shell completions

`mockers` can generate shell completion scripts for supported shells.  
To enable completions, add the following line to your shell startup file:

```sh
source <(mockers completion -s SHELL)
```

Replace SHELL with one of the supported shells:

- `bash`
- `zsh`
- `fish`
- `elvish`
- `powershell`

### Examples:

#### Bash:

Add this line to `~/.bashrc`:

```sh
source <(mockers completion -s bash)
```

#### Zsh

Add this line to `~/.zshrc`:

```sh
source <(mockers completion -s zsh)
```

#### Fish

Add this line to your Fish config file, usually ~/.config/fish/config.fish:

```sh
source (mockers completion -s fish | psub)
```

#### Elvish

Add this line to your Elvish config file, usually ~/.config/elvish/rc.elv:

```sh
eval (mockers completion -s elvish | slurp)
```

#### PowerShell

Add the generated script to your PowerShell profile:

```sh
mockers completion -s powershell | Out-String | Invoke-Expression
```

You can place this command in your PowerShell profile file so that completions are loaded automatically in every session.

---

## Shout-out

Huge thanks to [@Caik](https://github.com/Caik), whose [Go version](https://github.com/Caik/go-mock-server) sparked the original idea.
Also thanks to [bloodvez](https://github.com/bloodvez) and [silentroach](https://github.com/silentroach).

---

## License

MIT License.
