# Mockers — tiny but powerful HTTP mock server in Rust 🎯

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

- host: `127.0.0.1`
- port: `8080`
- mocks dir: `./mocks`

If `./mocks` does not exist, Mockers creates it.

---

## CLI commands overview

```bash
mockers <COMMAND>
```

Available commands:

- `serve` — run HTTP server
- `create` — create a file-based mock + config entry
- `list` — list all file-based mocks
- `info` — show full info for a mock (status, headers, delay, mime, body)
- `delete` — delete a specific mock (and clean config entry)
- `enable` — enable a disabled mock
- `disable` — disable a mock

There are also aliases (`run`, `start`, `ls`, `rm`, etc.), check `--help`.

---

## `serve` command

```bash
mockers serve [OPTIONS]
```

### Options

| Flag | Default | Description |
| --- | --- | --- |
| `--host` | `127.0.0.1` | Interface to bind to |
| `--port`, `-p` | `8080` | Port to listen on |
| `--mocks`, `-m` | `mocks` | Directory with mock files |
| `--cors`, `-c` | `false` | Adds CORS headers (`Access-Control-Allow-Origin: *`) |
| `--preflight` | unset | Auto-handle browser OPTIONS preflight requests |
| `--delay-ms`, `-d` | `0` | Global response delay in ms |
| `--origin`, `-o` | unset | Fallback upstream server when mock is missing |
| `--admin-base-url`, `-a` | unset | Enables admin API + Swagger under given absolute base path |
| `--log-request`, `-l` | `info` | Request logging level: `info`, `debug`, `trace` |

### Preflight modes

If `--preflight` is enabled, Mockers can answer browser preflight requests automatically:

- `mirror` — mirrors requested origin/method/headers back to the browser.
- `permissive` — basically "allow everything" mode (`*`, all methods, etc.).

Good for local dev when CORS fights you.

### Request logging levels

- `info`: method + path/query.
- `debug`: method + path/query + headers.
- `trace`: method + path/query + headers + body.

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

| Field | Type | What it does |
| --- | --- | --- |
| `delayMs` | number | Per-mock delay in ms |
| `statusCode` | number | Per-mock HTTP status |
| `headers` | object string->string | Extra response headers |
| `cacheMode` | `overwrite` \| `nocache` | Controls write-through behavior when proxying to `origin` |
| `disabled` | boolean | If `true`, file mock is ignored |

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
2. Try file-based mock.
3. If file mock is missing (or disabled):
   - if `--origin` is set -> proxy request to origin,
   - else -> return `404`.
4. If proxied and `cacheMode=overwrite`, write response to disk asynchronously (`mock file + config.json`).

So you can warm up mocks from a real backend automatically.

---

## `create` command

```bash
mockers create [OPTIONS] <ROUTE>
```

Creates:

- a mock file (`<route>.<method>`),
- and updates/creates `config.json` in the same directory.

### Options

| Flag | Default | Description |
| --- | --- | --- |
| `--method` | `GET` | HTTP method |
| `--status-code`, `-s` | `200` | Response status in config |
| `--delay-ms`, `-d` | `0` | Delay in config |
| `--header` | none | Add response header (`Key: Value`) |
| `--contents`, `-c` | `{"hello":"world"}` | Mock body content |
| `--cache-mode` | `nocache` | `overwrite` or `nocache` |
| `--mocks`, `-m` | `mocks` | Mocks directory |
| `--disabled` | `false` | Create mock as disabled |

### Example

```bash
mockers create /users/profile --method GET --status-code 200 --header "X-Env: local" --contents '{"id":1}'
```

---

## `list` command

```bash
mockers list [OPTIONS]
```

Shows all detected mock files with valid HTTP method extensions.

Example:

```bash
mockers list --mocks ./mocks
```

---

## `info` command

```bash
mockers info [OPTIONS] <NAME>
```

Searches by partial name/path (case-insensitive), then prints:

- normalized mock path,
- response status,
- configured headers,
- delay,
- detected mime type,
- body (`--show-body` only).

If multiple mocks match, it will ask you to be more specific by listing candidates.

---

## `delete` command

```bash
mockers delete [OPTIONS] <NAME>
```

Deletes one matching mock file.
Also removes that entry from `config.json` in the same directory.
If config becomes empty, `config.json` is deleted too.

If your query matches multiple mocks, it prints candidates and does nothing.

---

## `enable` / `disable` commands

```bash
mockers enable [OPTIONS] <NAME>
mockers disable [OPTIONS] <NAME>
```

These commands toggle the `disabled` flag in `config.json` for one matching mock.

- `disable` => mock is ignored at serve time.
- `enable` => mock becomes active again.

If multiple matches are found, it asks for a more specific query.

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
