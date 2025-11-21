# Mockers - Simple HTTP Mock Server in Rust

`Mockers` is a lightweight HTTP server written in Rust for serving mock responses from files. It is designed for testing, prototyping, or any scenario where you need a quick mock backend.

---

## Installation

Build from source using Cargo:

```bash
cargo build --release
```

## Usage

Run the server using the `serve` command:

```sh
mockers serve [OPTIONS]
```

## Command-line Options

| Flag              | Default     | Description                                            |
| ----------------- | ----------- | ------------------------------------------------------ |
| `--host`          | `127.0.0.1` | Host to listen on                                      |
| `--port`, `-p`    | `8080`      | Port to listen on                                      |
| `--verbose`, `-v` | `false`     | Enable verbose logging                                 |
| `--mocks`, `-m`   | `mocks`     | Path to the directory containing mock files            |
| `--cors`          | `false`     | Enable CORS headers (`Access-Control-Allow-Origin: *`) |
| `--delay-ms`      | `0`         | Delay (in milliseconds) for serving mock responses     |

## Mock File Structure

- All files inside the mocks directory are used as responses.
- File names determine the HTTP method:

```
user.get       -> responds to GET /user
login.post     -> responds to POST /login
```

- The path inside the file name (before the dot) corresponds to the URL path.
- Both relative and absolute paths are supported for the `--mocks` flag.

Example:

```
mocks/
├─ user.get
├─ login.post
└─ config.put
```

This will create the following endpoints:

- `GET /user`
- `POST /login`
- `PUT /config`

## Examples

Run the server on default settings:

```sh
mockers serve
```

Run on a custom host and port with verbose logging:

```sh
mockers serve --host 0.0.0.0 --port 3000 --verbose
```

Serve mocks from a custom directory with CORS enabled and 500ms response delay:

```sh
mockers serve --mocks ./api_mocks --cors --delay_ms 500
```

## Notes

- The server automatically resolves relative paths for mocks based on the current working directory.
- If the specified mocks directory does not exist or is not a directory, the server will return an error.
- Response delay can be used to simulate slow network responses.

## License

MIT License
