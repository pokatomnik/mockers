---
name: new-mock
description: Create or update one Mockers file-based mock by collecting endpoint path, HTTP method, exact body, and all per-mock config choices; writes the mock file and same-directory config.json using this skill's local config.v1.json.
---

# New Mock

Use this skill when the user wants to create or update one Mockers file-based mock.

The skill directory includes `config.v1.json`. Read and use that file as the only schema source for per-directory `config.json` files. Do not use any repository-level schema file.

## Runtime shape

A Mockers file-based mock consists of:

1. A mock data file named `<file-stem>.<http-method-lowercase>`.
2. A `config.json` file in the same directory as that mock data file.

For a request like `GET /users/profile`, Mockers looks for a file like:

```text
users/profile.get
users/config.json
```

The `config.json` entry key is the mock file name only:

```json
{
  "profile.get": {
    "statusCode": 200
  }
}
```

Do not include the directory path in the config key. Do not add `$schema` to `config.json`; it would be treated as a mock entry key.

In this skill, treat endpoint paths as paths under the opened project root, unless the user explicitly names a different target subdirectory inside the project. Never treat a leading `/` as an operating-system absolute path.

## Required workflow

Do not create or edit files until all ambiguity is resolved. If the user already provided some answers, validate them and ask only for the missing or invalid items.

### 0. Determine create vs update intent

Before writing, the operation mode must be clear:

- `create`: the user wants a new mock.
- `update`: the user wants to replace or adjust an existing mock/config entry.

You may collect endpoint, method, body, and config before checking the filesystem, but before writing you must derive the exact target paths and check whether the target mock file and target `config.json` entry already exist.

Overwrite rules:

- If the mock file exists, do not overwrite it unless the user explicitly confirms replacing that exact file after you show the derived path.
- If the target `config.json` entry exists, do not replace that entry unless the user explicitly confirms updating that exact entry key.
- If the user asked to create a new mock but the mock file or config entry already exists, stop and ask whether to update/replace it or choose a different endpoint/method.
- If the user asked to update a mock but the mock file and config entry are both missing, stop and ask whether to create it instead.

### 1. Endpoint path

Ask for the endpoint path without HTTP method.

Accepted examples:

- `/foo/bar/baz`
- `foo/bar/baz`

Interpret both as project-root-relative paths after stripping leading `/` characters.

Path parsing rules:

- Strip leading `/` characters.
- Reject an empty path after stripping.
- Split by `/`.
- Reject empty segments anywhere in the path.
- Reject path segments `.` or `..`.
- The last segment is the mock file stem.
- Everything before the last segment is the target directory under the project root.
- If there is no `/`, the target directory is the project root.
- If the user includes a query string, fragment, URL scheme, host, Windows path, backslashes, or HTTP method in the path, ask for a clean endpoint path.

Example:

User endpoint path: `/foo/bar/baz`

- Target directory: `foo/bar`
- File stem: `baz`

### 2. HTTP method

Ask for the HTTP method. This is mandatory.

Valid methods:

- `GET`
- `POST`
- `PUT`
- `PATCH`
- `DELETE`
- `HEAD`
- `OPTIONS`
- `CONNECT`
- `TRACE`

Normalize the method to lowercase for the file extension and config key.

Example:

- Endpoint path: `/foo/bar/baz`
- Method: `GET`
- Mock file path: `foo/bar/baz.get`
- Config path: `foo/bar/config.json`
- Config key: `baz.get`

Do not proceed without a valid method.

### 3. Existing target check

After deriving the target paths, check whether these already exist:

- the target mock data file;
- the same-directory `config.json`;
- the target config entry inside `config.json`, if that file exists and is valid JSON.

If `config.json` exists but is invalid JSON or is not a JSON object, stop. Do not rewrite it destructively.

Before any write, summarize:

- operation mode: create or update;
- mock file path;
- config path;
- config entry key;
- whether the mock file already exists;
- whether the config entry already exists.

If any existing mock file or entry will be replaced, get explicit confirmation.

### 4. Mock file content

Ask for the exact mock file content.

Rules:

- Write the content exactly as the user provides it unless they explicitly ask you to format it.
- Empty mock files are allowed only if the user explicitly confirms that the mock body must be empty.
- Do not invent mock content.
- Do not infer a response JSON shape unless the user asks you to draft it and then confirms the exact content to write.

### 5. Per-mock config fields

Ask about every schema field from the skill-local `config.v1.json`. The user must either provide a valid value or explicitly choose omission/default for each field.

You may ask about all fields in one structured prompt, but the final answers must cover each field individually:

- `delayMs`
- `statusCode`
- `headers`
- `cacheMode`
- `disabled`

If the user chooses to omit a field, do not write that field unless the user explicitly asks for an explicit default value. If all fields are omitted, write an empty object for the target entry: `{}`.

#### `delayMs`

Type: integer.
Range: `0..=18446744073709551615`.

Effect:

- Delay before returning a served file-based mock response, in milliseconds.
- If omitted, Mockers uses the effective global delay value.

Ask:

- “Should this mock set `delayMs`? If yes, provide a non-negative integer in milliseconds. If no, I will omit it and Mockers will use the effective global delay.”

#### `statusCode`

Type: integer.
Range: `100..=599`.

Effect:

- HTTP status code used when Mockers serves the mock file or a successful LLM prompt mock response.
- If omitted, Mockers uses `200`.

Ask:

- “Should this mock set `statusCode`? Choose an integer from 100 to 599, or say omit/default for 200.”

#### `headers`

Type: JSON object where every key is a string and every value is a string.

Effect:

- Adds custom response headers when Mockers serves this mock file or a successful LLM prompt mock response.
- Mockers auto-detects `Content-Type` from the response body first, then applies custom headers. Therefore an explicit `Content-Type` in `headers` overrides the auto-detected value for served mock responses.
- If `Content-Type` is omitted, Mockers keeps the auto-detected value.
- Successful proxy fallback responses use upstream headers; `headers` is not a proxy response rewrite mechanism.
- Runtime may ignore invalid HTTP header names or values, so prefer valid HTTP header syntax even though the schema only requires strings.

Ask:

- “Should this mock set response `headers`? Provide a JSON object of string-to-string headers, or say none/omit.”

Valid example:

```json
{
  "Content-Type": "application/json",
  "X-Mock": "true"
}
```

Invalid examples:

```json
{ "X-Count": 1 }
{ "X-Enabled": true }
```

#### `cacheMode`

Type: string enum.
Allowed values to write:

- `overwrite`
- `nocache`

Effect:

- `nocache`: do not persist proxied upstream responses.
- `overwrite`: only matters when a runtime request misses the mock file and `origin` proxy fallback is configured. In that case, Mockers may save the upstream response body as the missing mock file and write response metadata to the same-directory `config.json` entry.
- `cacheMode` does not affect normal serving when the mock file already exists.
- For a mock file created by this skill, `overwrite` is usually unnecessary for ordinary serving; set it only if the user explicitly wants future proxy write-through behavior.

Ask:

- “Choose `cacheMode`: `nocache`, `overwrite`, or omit/default. Use `overwrite` only if you want proxy fallback responses to be written as mocks when a mock file is missing.”

#### `disabled`

Type: boolean.

Effect:

- `false`: mock is active.
- `true`: a matching mock returns `404` immediately and the mock file is not served.
- If omitted, Mockers treats it as `false`.

Ask:

- “Should this mock be disabled? Answer `true`, `false`, or omit/default for active.”

## Config writing rules

The target `config.json` lives in the same directory as the mock file.

If `config.json` does not exist:

- Create it as a JSON object.
- Add one entry where the key is the mock file name, e.g. `baz.get`.

If `config.json` already exists:

- Read it first.
- It must be valid JSON with a top-level object.
- Preserve all unrelated existing entries exactly in meaning.
- Add or replace only the entry for the target mock file name.
- Keep the file valid JSON; prefer readable pretty formatting.
- If existing JSON is invalid or cannot be safely merged, stop and ask the user how to proceed.

Top-level entries other than the target key are other mocks. Do not delete or rename them. Unknown fields inside the target mock config are invalid; do not write them.

## File writing rules

- Create the target directory if it does not exist.
- Create all missing parent directories as needed.
- Use project-relative paths in tools.
- Never write outside the opened project root.
- Do not overwrite an existing mock data file or target config entry unless the overwrite rules above are satisfied.

## Mandatory checklist before tool use

Before creating or updating files, verify all items are true:

- The operation mode is clear: create or update.
- The user provided a valid endpoint path.
- The target directory is clear and is under the project root.
- The mock file stem is clear.
- The HTTP method is clear and valid.
- The mock file path is clear and ends with a lowercase valid HTTP method extension.
- The config path is clear and is in the same directory as the mock file.
- The mock content is clear, or the user explicitly confirmed an empty mock file.
- Every config field from the skill-local `config.v1.json` was discussed.
- Every provided config value is valid according to the skill-local `config.v1.json`.
- The final `config.json` entry key is the mock file name only, without path.
- Existing file/entry status was checked.
- Any replacement of an existing mock file or config entry was explicitly confirmed.

Do not proceed if any checklist item is unresolved.

## Final response

After writing files, report:

- mock file path;
- `config.json` path;
- config entry key;
- whether this was a create or update;
- whether any existing file or entry was replaced;
- config fields written;
- omitted fields and their runtime defaults/effects.
