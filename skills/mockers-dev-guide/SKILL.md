---
name: mockers-dev-guide
description: Modify the Mockers Rust codebase safely. Use when the user asks to change server behavior, add commands, edit handlers, middleware, or tests.
---

# Mockers developer guide

Use this skill when the user wants to change the Mockers codebase itself.

## What to do

1. Read the request as an implementation task, not as a usage question.
2. Locate the relevant area in `src/`:
   - CLI and commands in `src/cmd/`
   - server runtime in `src/server/`
   - request handlers in `src/controllers/`
   - middleware in `src/middlewares/`
   - shared helpers in `src/libs/`
3. Follow the project’s patterns:
   - prefer extension traits over newtypes for foreign types
   - keep async I/O async
   - use `anyhow::Result` for fallible functions
   - keep config structs builder-friendly when that matches the surrounding code
   - add inline tests near the code under test when appropriate
4. Keep changes surgical and aligned with existing conventions.
5. If the user asks for a new CLI command or a new server feature, update the relevant command plumbing, config surfaces, docs, and tests together.
6. Validate with the narrowest useful diagnostics or test run after making changes.

## Important conventions

- Do not invent new patterns if the repository already has an established one.
- Preserve the CLI naming and serde naming conventions used in the project.
- Treat the project docs and `AGENTS.md` as the source of truth for architecture and behavior.
- If the change affects user-facing behavior, call out the impact clearly.

## Common checks

- Check for related config and routing changes, not just the primary handler.
- Check whether a change needs a matching doc update.
- Check diagnostics after edits if the language tooling is available.
