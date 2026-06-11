# AGENTS.md — Mockers Codebase Guide for AI Agents

## Project Overview

Mockers is a CLI + HTTP mock server written in Rust. It serves file-based mocks,
proxies to upstream origins when mocks are missing, supports admin API + Swagger UI,
HTTPS, CORS/preflight handling, and hierarchical global configuration.

---

## Project Structure

```
src/
├── main.rs                    # Entry point: logger init, Cli::parse(), dispatch
├── cmd/                       # CLI layer — clap subcommands
│   ├── cli.rs                 # Cli struct (clap Parser)
│   ├── commands.rs            # Commands enum + aliases (Serve, Create, List, etc.)
│   └── mod.rs
├── server/                    # HTTP server runtime
│   ├── params.rs              # ServerParams — all serve CLI options + constants
│   ├── mockers_router.rs      # Router construction using routerify_ng
│   ├── mockers_context.rs     # MockersContext — shared state (reqwest Client + params)
│   ├── route_error.rs         # MockersRouteError enum
│   ├── signal.rs              # Graceful shutdown (Ctrl+C)
│   ├── get_info_async.rs      # GetInfoAsync trait
│   ├── banner.txt             # ASCII art banner
│   └── mod.rs
├── controllers/               # Request handlers
│   ├── mock_handler.rs        # Main request handler (file mock / proxy)
│   ├── error.rs               # Error handler
│   ├── mod.rs
│   ├── entities/              # Request/response DTOs
│   │   ├── mock_create_params.rs
│   │   ├── mock_info.rs
│   │   └── empty.rs
│   ├── api/                   # Admin API handlers
│   │   ├── admin_page_handler.rs
│   │   ├── create_mock.rs
│   │   ├── delete_mock.rs
│   │   ├── get_all_mocks.rs
│   │   ├── get_mock_config.rs
│   │   ├── handle_options.rs
│   │   └── mod.rs
│   └── swagger/               # Swagger UI static file handlers
│       ├── static_files.rs
│       ├── get_swagger_html.rs, get_swagger_json.rs, etc.
│       └── mod.rs
├── middlewares/                # Router middlewares
│   ├── logger.rs              # Request logging (VerbosityLevel)
│   ├── check_request.rs       # HTTP method validation
│   ├── admin_api_cors.rs      # CORS headers for admin API
│   └── mod.rs
└── libs/                      # Shared libraries and utilities
    ├── absolute_mocks_path.rs # AbsoluteMocksPath trait + path resolution
    ├── cache_mode.rs          # CacheMode enum (Overwrite, NoCache)
    ├── config.rs              # ConfigParams for `mockers config` command
    ├── create_params.rs       # CreateParams for `mockers create` command
    ├── delete_params.rs       # DeleteParams for `mockers delete` command
    ├── info_params.rs         # InfoParams for `mockers info` command
    ├── ls_params.rs           # LsParams for `mockers list` command
    ├── activity_params.rs     # ActivityParams for enable/disable
    ├── init_params.rs         # InitParams for `mockers init` command
    ├── doc_params.rs          # DocParams for `mockers doc` command
    ├── completion_params.rs   # CompletionParams for shell completions
    ├── global_config.rs       # GlobalConfig — hierarchical .mockers config
    ├── mock_config.rs         # MockConfig — per-mock config.json entry
    ├── preflight_type.rs      # PreflightType enum (Mirror, Permissive)
    ├── http_method.rs         # HTTP method validation traits
    ├── get_mime.rs            # MIME detection with LRU cache
    ├── fs_walker.rs           # Recursive filesystem walker
    ├── fs_cached_reader.rs    # In-memory cached file reader
    ├── mockers_errors.rs      # MockersErrors enum for API responses
    ├── protocol_result.rs     # ProtocolResult<T,E> for HTTP serialization
    ├── response_builder_ext.rs # Response builder extension trait
    ├── hyper_response_ext.rs  # Well-known HTTP response constructors
    ├── reqwest_response_ext.rs # Reqwest response body reader with cap
    ├── mockers_request_ext.rs # Request extensions (preflight detection, body read)
    ├── header_map_ext.rs      # HeaderMap conversion/sanitization
    ├── url_ext.rs             # URL construction from parts
    ├── path_buf_ext.rs        # PathBuf helpers (last component removal)
    ├── status_code_ext.rs     # Status code listing/pretty-print
    ├── tls_acceptor_ext.rs    # TLS acceptor loader (cert.pem + key.pem)
    ├── tap.rs                 # Tap trait (similar to tap in Ruby)
    ├── headers.rs             # CORS header constants
    └── mod.rs
```

---

## Key Architecture Patterns

### 1. Trait-based Extension Pattern

Mockers heavily uses **extension traits** to add methods to foreign types instead of wrapping them:

- `Tap` trait — adds `.tap(|x| ...)` to any `Sized` type for inline mutation
- `ResponseBuilderExt` — adds `.add_cors()`, `.add_custom_headers()`, `.empty_body()` to `http::response::Builder`
- `HyperWellKnownResponses` — adds `.not_found()`, `.bad_gateway()`, etc. to `Response`
- `HyperHTTPMethodExt`, `StandardMethodValidator` — adds validation to `hyper::Method`
- `HeaderMapConverter`, `HeaderMapSanitizer` — adds conversion to `HeaderMap`
- `AbsoluteMocksPath`, `WithMocks` — path resolution traits

**Rule:** When you need to add behavior to a foreign type, prefer an extension trait over a newtype wrapper.

### 2. File-based Mock Resolution

The mock handler (`controllers/mock_handler.rs`) follows this flow:

1. Build relative file path from URL path + HTTP method: `/users/profile` + `GET` → `users/profile.get`
2. Append to absolute mocks directory
3. Read `config.json` from the same directory as the mock file
4. Check `disabled` flag, delay, status code, headers, cache mode from config
5. If file exists and is not disabled → serve file with config overrides
6. If file missing/disabled and `--origin` is set → proxy request to upstream
7. If `cacheMode=overwrite` during proxy → save response to disk asynchronously

### 3. Global Config Resolution (`~/.mockers`)

`GlobalConfigAPI` reads `.mockers` files from CWD up to filesystem root, merges them.
CLI flags override global config, which overrides built-in defaults.

Resolution order: `CLI args > .mockers (closest first) > hardcoded defaults`

### 4. Router Pipeline

`routerify_ng` middleware pipeline order:

```
logger (pre) → check_request (pre) → mock_handler (catch-all) → error_handler
```

Admin API is mounted as a scope under `--admin-base-url` with its own CORS middleware.

---

## Naming Conventions

| Context | Convention | Example | Source |
|---------|-----------|---------|--------|
| CLI args | `kebab-case` | `--admin-base-url`, `--delay-ms` | `clap(rename_all = "kebab-case")` |
| JSON keys (config.json, .mockers) | `camelCase` | `delayMs`, `adminBaseUrl`, `proxyBodyMaxBytes` | `serde(rename_all = "camelCase")` |
| API routes | kebab-case path | `/api/v2/mocks/create` | router definition |
| API error codes | `SCREAMING_SNAKE_CASE` | `INTERNAL_SERVER_ERROR` | `MockersErrors` enum |
| Mock file extensions | lowercase HTTP method | `profile.get`, `login.post` | file-system convention |
| Enum variants (serde) | `lowercase` | `"mirror"`, `"permissive"`, `"overwrite"` | `serde(rename_all = "lowercase")` |
| Enum variants (clap) | `lowercase` | `mirror`, `permissive`, `overwrite` | `clap(rename_all = "lowercase")` |

---

## Important Types

### MockConfig (per-mock `config.json`)
```rust
pub struct MockConfig {
    delay_ms: Option<u64>,
    status_code: Option<u16>,
    headers: Option<HashMap<String, String>>,
    cache_mode: Option<CacheMode>,  // Overwrite | NoCache
    disabled: Option<bool>,
}
```
Uses builder pattern: `MockConfig::new().with_delay_ms(100).with_status_code(201)`.

### GlobalConfig (`~/.mockers`)
```rust
pub struct GlobalConfig {
    host, port, https_port, mocks, cors, preflight, delay_ms,
    origin, admin_base_url, log_request, verbosity,
    proxy_body_max_bytes, proxy  // all Option<T>
}
```
Same builder pattern. Merging logic: `merge(&self, other)` — `other` fields override `self` fields.

### CacheMode
```rust
pub enum CacheMode {
    Overwrite,  // Persist proxied responses to disk
    NoCache,    // Do not cache (default)
}
```

### PreflightType
```rust
pub enum PreflightType {
    Mirror,      // Echo requested origin/method/headers
    Permissive,  // Allow everything
}
```

### VerbosityLevel
```rust
pub enum VerbosityLevel {
    Info,   // method + URI
    Debug,  // + headers
    Trace,  // + body
}
```

### MockersContext
```rust
pub struct MockersContext {
    pub server_params: ServerParams,
    pub client: Arc<Client>,  // reqwest::Client (with or without proxy)
}
```
Shared state injected into the router via `routerify_ng`'s `data()` mechanism.

### ProtocolResult<T, E>
```rust
pub enum ProtocolResult<T, E> {
    Ok(T),
    Err(E),
}
```
Used for HTTP API responses — serializes to `{"Ok": ...}` or `{"Err": ...}` in camelCase.

---

## Module Relationships

```
main.rs
 └─ Cli::parse()
     └─ Commands::Serve  → ServerParams.test() → ServerParams.start_server()
     └─ Commands::Create → CreateParams.test() → CreateParams.create_mock()
     └─ Commands::List   → LsParams.ls_mocks()
     └─ Commands::Info   → InfoParams.show_info()
     └─ Commands::Delete → DeleteParams.delete_mock()
     └─ Commands::Enable/Disable → ActivityParams.enable()/disable()
     └─ Commands::Config → ConfigParams.show_config()
     └─ Commands::Init   → InitParams.init()
     └─ Commands::Doc    → DocParams.show_help()
     └─ Commands::Completion → CompletionParams.generate()

start_server():
 └─ create TCP listener (host:port)
 └─ build router (mockers_router):
     ├─ admin_router (scope under admin_base_url)
     ├─ middlewares: logger, check_request
     └─ catch-all handler: mock_handler
 └─ optionally build TLS acceptor (cert.pem + key.pem)
 └─ enter accept loop with graceful shutdown
```

---

## Code Style Guidelines

1. **Error handling:** Use `anyhow::Result` for fallible functions. Convert lib errors via `map_err(anyhow::Error::from)` or `?`.

2. **Async everywhere:** All I/O and handlers are async. Use `tokio::join!` / `tokio::try_join!` for concurrent operations.

3. **Builder pattern** for config-like structs: methods return `self` to enable chaining.

4. **`Tap` trait** for inline mutations without `let mut`:
   ```rust
   // Instead of:
   let mut builder = Response::builder().status(200);
   builder = builder.add_cors();
   
   // Use:
   Response::builder()
       .status(200)
       .tap(|b| if cors { b.add_cors() } else { b })
   ```

5. **Extension traits** over newtypes (see section 1).

6. **`OnceCell`** for lazy initialization of shared state in CLI param structs.

7. **`LazyLock`** for statics that need initialization (e.g., MIME cache).

8. **Constants** for defaults live in `server/params.rs` (e.g., `DEFAULT_PORT`, `DEFAULT_MOCKS_DIR_NAME`).

9. **`serde(rename_all = "camelCase")`** for all config/serialization structs.

10. **`#[clap(rename_all = "kebab-case")]`** for CLI argument structs.

---

## Adding a New CLI Command

1. Add a `Params` struct in `src/libs/<name>_params.rs` with `#[derive(clap::Args)]`
2. Add the command variant to `Commands` enum in `src/cmd/commands.rs`
3. Add match arm in `src/main.rs`
4. If the command needs mocks directory → implement `WithMocks` and `WithGlobalConfigAPI` traits
5. Add corresponding doc file in `doc/<name>.md`
6. Add doc entry in `doc_params.rs`'s `get_markdown_by_kind()`

**Traits for commands that use mocks:**
```rust
impl WithMocks for MyParams {
    fn get_mocks(&self) -> Option<&str> {
        self.mocks.as_ref().map(|x| x.as_str())
    }
}

impl WithGlobalConfigAPI for MyParams {
    async fn get_global_config(&self) -> &GlobalConfigAPI {
        self.global_config.get_or_init(GlobalConfigAPI::new).await
    }
}
```

---

## Adding a New Server Feature

1. Add CLI option to `ServerParams` in `src/server/params.rs` (use `Option<T>`)
2. Add runtime default to `server/params.rs` constants section
3. Add getter method to `ServerParams` (async, with global config fallback)
4. Add field to `GlobalConfig` in `src/libs/global_config.rs`
5. Wire into `mock_handler` or relevant middleware
6. Handle in `admin_router` / API controllers if applicable

---

## Testing Patterns

- Inline `#[cfg(test)] mod tests { ... }` in the same file as the code under test (not separate files)
- Use `#[tokio::test]` for async tests
- Use helper functions with `#[cfg(not(windows))]` / `#[cfg(windows)]` for cross-platform path tests
- Example from `absolute_mocks_path.rs`:
  ```rust
  #[cfg(not(windows))]
  fn get_current_dir() -> anyhow::Result<PathBuf> {
      Ok("/home/john_doe".into())
  }
  ```
- Tests are in the library crates or inline, not in a separate `tests/` directory
- MIME detection tests demonstrate testing with raw byte slices

---

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `hyper` | HTTP server implementation |
| `routerify_ng` | Routing and middleware pipeline |
| `reqwest` | HTTP client for proxying to upstream |
| `clap` + `clap_complete` | CLI argument parsing + shell completions |
| `serde` + `serde_json` | Serialization (config, API responses) |
| `tokio` + `tokio-rustls` | Async runtime + TLS |
| `dialoguer` | Interactive prompts (init, create) |
| `mimetype-detector` + `raffia` | MIME type detection (JSON/CSS detection) |
| `lru` + `xxhash-rust` | MIME cache |
| `termimad` | Terminal markdown rendering (doc command) |
| `path-absolutize` | Path normalization |
| `simplelog` | Logging |

---

## Common Pitfalls

1. **`mock_config_entry_name` is the file name** (e.g., `profile.get`), not the normalized route. This is because `config.json` keys are file names, not URL paths.

2. **`config.json` is directory-local** — each directory with mocks can have its own `config.json`. It only affects mocks in that same directory.

3. **Path resolution for `--mocks`** — supports absolute paths, relative paths (resolved from CWD), and `~` prefix (expanded to home directory). All resolved via `generic_get_absolute_mocks_path`.

4. **`admin_base_url` must be an absolute path** (start with `/`). Validation in `ServerParams::check_admin_base_url`.

5. **`proxy_body_max_bytes` is hard-capped** at `HARD_MAX_PROXY_RESPONSE_BODY_BYTES` (32 MiB = 2 × default). Values above are clamped.

6. **Global config merge** — closer `.mockers` file overrides parent. `origin` and `proxy` are not set by default in `init`.

7. **`cacheMode: overwrite`** — only takes effect when proxying to `--origin`. Writes mock file + config.json asynchronously via `tokio::spawn`.

8. **Graceful shutdown** — handled via `tokio::signal::ctrl_c()`. The server stops accepting new connections after SIGINT.

9. **CORS is not preflight** — `--cors` adds `Access-Control-Allow-Origin: *` to regular responses; `--preflight` handles OPTIONS requests independently. Use both for full browser interop.

10. **MIME detection order**: JSON (serde) → signature-based → CSS parser → fallback to `text/utf-8`. Uses LRU cache (1024 entries, XXH3-128 hashing).
