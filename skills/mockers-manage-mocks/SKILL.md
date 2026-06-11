---
name: mockers-manage-mocks
description: List, inspect, enable, disable, and delete existing Mockers mocks. Use when the user wants to understand or change existing mock files rather than create a new one.
---

# Mockers manage mocks

Use this skill when the user wants to work with existing mocks.

## What to do

1. Determine the operation:
   - list mocks
   - inspect a mock
   - delete a mock
   - enable a mock
   - disable a mock
2. Use the file-system conventions to find the correct mock file:
   - URL path + HTTP method maps to a file like `users/profile.get`
3. If the user says a mock is not being served, check the likely causes in order:
   - wrong file path or method extension
   - missing mock file
   - local `config.json` has `disabled: true`
   - proxy fallback is taking over because `--origin` is enabled
4. When explaining a result, mention both the mock file and the local directory config if relevant.
5. Prefer editing the smallest necessary file rather than rewriting whole directories.

## Important conventions

- Deleting a mock should remove the file only when that is what the user asked; otherwise disabling via config is safer.
- Treat `config.json` as directory-local metadata, not a global setting.
- Keep route mapping and method matching explicit when diagnosing problems.

## Common checks

- Confirm the mock path matches the intended request path.
- Confirm the method extension is correct and lowercase.
- Confirm no local config is disabling the mock.
- Confirm the request is not being proxied to an origin instead.
