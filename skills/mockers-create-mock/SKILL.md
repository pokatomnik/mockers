---
name: mockers-create-mock
description: Create and edit file-based HTTP mocks in Mockers. Use when the user wants a new mock, to change a response body/status/headers/delay, or to enable/disable an existing mock.
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
3. Create or update the mock file content.
4. If the mock needs metadata, create or update the adjacent `config.json` in the same directory.
5. Keep config keys in `camelCase`:
   - `delayMs`
   - `statusCode`
   - `headers`
   - `cacheMode`
   - `disabled`
6. Remember that `config.json` is directory-local and affects only mocks in that directory.
7. If the user wants to manage the mock through the admin API instead of the file system, use the admin API flow from the project docs.

## Important conventions

- Use lowercase HTTP method extensions for mock files.
- Keep route-to-file mapping predictable and minimal.
- Prefer the simplest mock that satisfies the user request.
- If the user says a mock should be disabled, set `disabled: true` in the local config rather than deleting the file.

## Common checks

- Verify the mock path matches the intended URL and method.
- Verify the status code, headers, and delay match the requested behavior.
- If the user asks for caching behavior with proxy fallback, make sure `cacheMode` is set intentionally.
