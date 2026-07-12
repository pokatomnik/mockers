# AGENTS.md — Mockers Codebase Guide for AI Agents

## Purpose

Mockers is a Rust CLI + HTTP mock server with file mocks, proxy fallback, admin API, Swagger UI, HTTPS, CORS/preflight, and hierarchical config. Keep changes aligned with this architecture.

## Layers

Higher layers may depend only on lower layers.

- `main.rs` — entrypoint, logger init, CLI dispatch.
- `cli` — CLI parsing and CLI controllers.
- `server` — HTTP runtime, router, middleware, HTTP controllers, server-specific helpers.
- `use_cases` — project-specific scenario logic, not generic infra.
- `entities` — domain data, config models, enums.
- `libs` — domain-agnostic helpers and extension traits.

## Dependency Rules

- `main.rs` may use all lower layers.
- `cli` may use `entities`, `use_cases`, `libs`; it must not couple to server internals.
- `server` may use `entities`, `use_cases`, `libs`.
- `use_cases` may use `entities`, `libs`.
- `entities` must not depend on `cli` or `server`.
- `libs` must stay generic.
- Reusable across projects → `libs`.
- Mockers-specific → `entities` or `use_cases`.

## Runtime Shape

- CLI: `main.rs` parses args and dispatches to CLI controllers.
- HTTP: `logger -> request validation -> main mock handler -> error handler`.
- Admin API is mounted separately and may use its own middleware.
- Mock resolution: request → mock file name → mocks dir → per-dir `config.json` → serve → proxy if configured → optional cache.
- Global config is hierarchical: closer `.mockers` overrides parent; CLI flags override config defaults. Use **effective config** for merged output.

## Architectural Patterns

- Use extension traits for foreign types.
- Use builder-style config structs (`Self`-returning setters).
- Async by default; use `tokio` primitives when coordination is needed.
- Use `OnceCell` for lazy CLI-param state; use `RwLock` only for cached shared state.

## Naming

- CLI flags: `kebab-case`
- JSON/YAML config keys: `camelCase`
- Serde enum strings: lowercase
- Mock files: lowercase HTTP method suffixes, e.g. `profile.get`
- API routes: kebab-case
- Error codes: `SCREAMING_SNAKE_CASE`

## Implementation Rules

1. Keep routing separate from controllers/handlers.
2. Keep CLI and server concerns isolated.
3. Move domain-specific types out of `libs`.
4. Keep `use_cases` for project-specific behavior, not generic infra.
5. Do not change runtime behavior just to satisfy refactors unless asked.
6. Keep edits minimal and style-consistent.
7. Update tests when behavior changes.
8. Do not delete meaningful code just to silence warnings.
9. `unwrap` is forbidden outside tests.
10. In production code, use `match`, `if let`, or explicit error propagation.
11. `expect` is allowed only with a strong documented justification; ask for confirmation before using it.

## Testing

- Use inline unit tests in the same file as the code under test.
- Use `#[tokio::test]` for async tests.
- Write pure unit tests.
- Any I/O is forbidden in unit tests: filesystem, network, env access, process interaction, and similar side effects.
- Use cross-platform helpers for path tests.

Keep the architecture layered: upper layers may depend only on lower layers.
