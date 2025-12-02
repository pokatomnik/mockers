# Mockers - Simple HTTP Mock Server in Rust 🎯

[![Rust](https://github.com/pokatomnik/mockers/actions/workflows/rust.yml/badge.svg)](https://github.com/pokatomnik/mockers/actions/workflows/rust.yml)

`Mockers` is a lightweight HTTP server written in Rust for serving mock responses from files. It is designed for testing, prototyping, or any scenario where you need a quick mock backend.

---

## Installation 🚀

Build from source using Cargo:

```bash
cargo build --release
```

## Usage 🚀

Run the server using the `serve` command:

```sh
mockers serve [OPTIONS]
```

## Command-line Options 🚀

| Flag              | Default     | Description                                            |
| ----------------- | ----------- | ------------------------------------------------------ |
| `--host`          | `127.0.0.1` | Host to listen on                                      |
| `--port`, `-p`    | `8080`      | Port to listen on                                      |
| `--verbose`, `-v` | `false`     | Enable verbose logging                                 |
| `--mocks`, `-m`   | `mocks`     | Path to the directory containing mock files            |
| `--cors`          | `false`     | Enable CORS headers (`Access-Control-Allow-Origin: *`) |
| `--delay-ms`      | `0`         | Delay (in milliseconds) for serving mock responses     |
| `--origin`        | [unset]     | Forward requests to another server when mock is missing by requested URL |

## Mock File Structure 🚀

- 💡 All files inside the mocks directory are used as responses.
- 💡 File names determine the HTTP method:

```
user.get       -> responds to GET /user
login.post     -> responds to POST /login
```

- 💡 The path inside the file name (before the dot) corresponds to the URL path.
- 💡 Both relative and absolute paths are supported for the `--mocks` flag.

Example:

```
mocks/
├─ user.get
├─ login.post
└─ config.put
```

This will create the following endpoints:

- 💡 `GET /user`
- 💡 `POST /login`
- 💡 `PUT /config`

## Examples 🚀

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

## Per-Directory Mock Configuration 🚀

Some endpoints may require custom behavior — a delayed response, a non-200 status code, or custom headers.
To support this, any mock directory may optionally contain a mock-config.json file describing additional response parameters.

### Example 🔧

```json
{
  "test.get": {
    "delayMs": 5000,
    "statusCode": 201,
    "headers": {
      "X-Server": "Mockers"
    }
  }
}
```

Given the file above, a request to:

```
GET http://localhost:8080/test
```

will produce:

- 💡 **5000 ms delay**
- 💡 **HTTP 201 status**
- 💡 **Header** `X-Server: Mockers`
- 💡 **Body** — the content of test.get (or any corresponding mock file)

### Rules 🔧

- 💡 The config file is optional.
- 💡 If it doesn't exist, default behavior applies (status 200, no delay, no custom headers).
- 💡 Keys in the config file must match mock filenames in the same directory.
- 💡 For example, test.get configures the file test.get.
- 💡 All fields inside each entry are optional:

| Field        | Type                    | Description                                  |
| ------------ | ----------------------- | -------------------------------------------- |
| `delayMs`    | `number`                | Artificial response delay in milliseconds    |
| `statusCode` | `number` (u16)          | HTTP status code                             |
| `headers`    | `Record<string,string>` | Additional headers to append to the response |

### Example Behavior 🔧

If only some fields are provided, the server fills in the rest with defaults.
For example:

```json
{
  "user.get": {
    "statusCode": 404
  }
}
```

This results in:

- 💡 404 status
- 💡 no delay
- 💡 no custom headers
- 💡 body loaded from `user.get`

## Notes 🚀

- 💡 The server automatically resolves relative paths for mocks based on the current working directory.
- 💡 If the specified mocks directory does not exist or is not a directory, the server will return an error.
- 💡 Response delay can be used to simulate slow network responses.

## Shout-out 🚀

Huge thanks to [@Caik](https://github.com/Caik)
, whose [Go version](https://github.com/Caik/go-mock-server) sparked the idea for this project.
I rewrote the whole thing in Rust because apparently I enjoy suffering — and because I wanted features the original never asked for.

Special thanks to [bloodvez](https://github.com/bloodvez) who helped me with finding issues.

## License 🚀

MIT License
