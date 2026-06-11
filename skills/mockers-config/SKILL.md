---
name: mockers-config
description: Work with Mockers configuration files and precedence, including `.mockers`, per-mock `config.json`, defaults, cache mode, delay, headers, status, and disabled flags.
---

# Mockers config

Use this skill when the user asks about configuration in Mockers.

## What to do

1. Distinguish the scope of the setting:
   - global `.mockers`
   - per-mock `config.json`
   - CLI flag
   - built-in default
2. Apply precedence correctly:
   - CLI overrides `.mockers`
   - `.mockers` from the closest directory wins over parent directories
   - per-mock `config.json` applies only to mocks in the same directory
3. Keep config keys in `camelCase`.
4. When the user asks about proxy caching, remember that `cacheMode: overwrite` only matters when proxying to an origin.
5. When the user asks about delays, headers, or status codes, put the setting in the smallest scope that actually needs it.
6. If a setting looks wrong, explain whether the source of truth is the CLI, the global config, or the local mock config.

## Important conventions

- `.mockers` is hierarchical and merges from parent directories to the current directory.
- `config.json` is stored next to the mock file or in the mock directory.
- Use `delayMs`, `statusCode`, `headers`, `cacheMode`, and `disabled` as the config surface.
- Be explicit about whether the change should affect one mock, one directory, or the whole run.

## Common checks

- Confirm which layer currently owns the value.
- Confirm whether the user wants a permanent config change or only a one-off CLI override.
- Confirm whether proxying is enabled before suggesting cache mode changes.
