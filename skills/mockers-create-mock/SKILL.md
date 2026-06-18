---
name: mockers-create-mock
description: Create and edit file-based HTTP mocks in Mockers. Use when the user wants a new mock, an LLM prompt mock, to change a response body/status/headers/delay, or to enable/disable an existing mock.
---

# Mockers create mock

Use this skill when the user asks to create or edit a mock in a Mockers repository.

## What to do

1. Ask for any missing required details:
   - HTTP method
   - URL path
   - response body or file content
   - target mocks directory if it is not obvious
2. Map the request to the file-system convention:
   - `/users/profile` + `GET` -> `users/profile.get`
   - the file extension is the lowercase HTTP method
3. Decide whether this is a static mock or an LLM prompt mock:
   - static mock: file content is the exact response body,
   - LLM prompt mock: file content starts with Mockers YAML frontmatter and then contains the prompt.
4. Create or update the mock file content.
5. If the mock needs metadata, create or update the adjacent `config.json` in the same directory.
6. Keep `config.json` keys in `camelCase`:
   - `delayMs`
   - `statusCode`
   - `headers`
   - `cacheMode`
   - `disabled`
7. Remember that `config.json` is directory-local and affects only mocks in that directory.
8. If the user wants to manage the mock through the admin API instead of the file system, use the admin API flow from the project docs.

## Important conventions

- Use lowercase HTTP method extensions for mock files.
- Keep route-to-file mapping predictable and minimal.
- Prefer the simplest mock that satisfies the user request.
- If the user says a mock should be disabled, set `disabled: true` in the local config rather than deleting the file. A disabled mock returns `404` immediately and does not fall back to `--origin`.

## LLM prompt mock frontmatter

Use this only when the user asks for an LLM/generated/dynamic mock or explicitly provides prompt/provider settings.

LLM prompt mock files must be UTF-8 text and start with YAML frontmatter:

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
Return a realistic JSON response for this request.
```

Rules:

- Frontmatter must be at the very beginning of the file.
- Opening and closing delimiters must be exact `---` lines.
- The Mockers-specific key is literal `$mockers`.
- `$mockers.prompt: true` is required to trigger LLM generation.
- Field names inside `$mockers` are snake_case where applicable: `api_endpoint`, `env_key`.
- `api_endpoint` and `model` are required for an enabled prompt mock.
- `env_key` is optional and names the environment variable that contains the bearer token.
- `proxy` is optional and applies only to the LLM provider request.
- `ttl` is optional and is a cache TTL in milliseconds.
- If prompt frontmatter is invalid, missing, or has `prompt: false`, Mockers serves the file as a static mock.
- The generated response still uses `statusCode` and `headers` from adjacent `config.json`.

## Common checks

- Verify the mock path matches the intended URL and method.
- Verify the status code, headers, and delay match the requested behavior.
- If the user asks for caching behavior with proxy fallback, make sure `cacheMode` is set intentionally.
