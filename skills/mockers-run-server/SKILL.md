---
name: mockers-run-server
description: Plan and launch Mockers server configurations, including mocks directory, origin fallback, CORS, preflight, HTTPS, logging, Docker, and shell completions.
---

# Mockers run server

Use this skill when the user wants to start Mockers with a particular runtime setup.

## What to do

1. Identify the requested runtime shape:
   - mocks directory
   - host and port
   - origin fallback
   - CORS
   - preflight behavior
   - HTTPS
   - request logging / verbosity
   - Docker or bare-metal execution
2. Resolve configuration precedence:
   - CLI flags win over `.mockers`
   - `.mockers` wins over built-in defaults
3. Apply path rules for `--mocks`:
   - absolute path
   - relative path from the current working directory
   - `~` expansion to the home directory
4. If the user asks for fallback to a real backend, wire `--origin` accordingly.
5. If the user asks for browser access, distinguish:
   - `--cors` for `Access-Control-Allow-Origin: *`
   - `--preflight` for OPTIONS handling
   Use both when the frontend needs both regular CORS and preflight support.
6. If the user asks for HTTPS, follow the project’s TLS/certificate setup from the docs.
7. If the user asks for Docker, use the README’s Docker flow and explain port and volume mapping.
8. If the user asks for completions, point them to the shell-completion command flow.

## Important conventions

- Keep the answer operational: give the exact command shape the user should run.
- Prefer the smallest working set of flags.
- Call out when a setting comes from `.mockers` versus CLI.
- Mention that graceful shutdown is handled by Ctrl+C.

## Common checks

- Verify the chosen mocks directory exists and is the one the user expects.
- Verify origin fallback is only enabled when the user explicitly wants proxying.
- Verify HTTPS requirements before claiming the setup is complete.
