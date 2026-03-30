# Help

## Mockers — tiny but powerful HTTP mock server in Rust 🎯

Mockers is a CLI + HTTP server that lets you spin up fake APIs from files in seconds.
If you need a quick backend for frontend work, contract testing, demos, QA, or local integration tests — this thing is exactly for that. No heavy setup, no DB, no headache.

---

## Why this project exists

You have requests coming in.
You want deterministic responses.
You want them fast.

Mockers maps URL + HTTP method to files, returns file contents as responses, and gives you extra controls like:

- custom status codes,
- response delay,
- custom headers,
- admin API,
- request forwarding to a real origin,
- optional disk write-through caching.

So it can work as both:

- a pure mock server, and
- a "mock-first, proxy-if-missing" server.
