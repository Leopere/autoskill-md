# autoskill-md

`autoskill-md` writes `.well-known/skills.md` for an API or repo.

Agents read that file first. It tells them what the API does. It also tells them what is safe.

This project follows the Agent Skill Discovery spec by Colin Knapp:

https://colinknapp.com/specs/mcp-discovery.html

## Install

```sh
npm install --save-dev autoskill-md
```

## Use

Create or update the skills file:

```sh
npx autoskill-md generate
```

Check it in CI:

```sh
npx autoskill-md check --strict
```

The default output is:

```text
.well-known/skills.md
```

## Config

Run this once:

```sh
npx autoskill-md init
```

Then edit `autoskill.config.json`.

```json
{
  "name": "my-api",
  "purpose": "This API lets agents read ticket status.",
  "apiBase": "/api",
  "auth": "Use a bearer token for private calls.",
  "safeActions": ["GET ticket status"],
  "riskyActions": ["Ask before write or delete calls"],
  "docs": ["https://example.com/docs"],
  "support": "https://example.com/support",
  "limits": "Use a slow pace.",
  "ignore": []
}
```

## Code Hints

Add short comments near routes or modules.

```js
// autoskill: purpose: This API lets agents read ticket status.
// autoskill: safe: GET ticket status.
// autoskill: risky: Ask before changing a ticket.
```

The scanner also looks at common route code in Go, Rust, Node.js, and Python.

## Build Use

This tool is best effort by default. It should not slow normal builds.

Use `generate` in local scripts. Use `check --strict` only when you want CI to fail on stale output, secrets, or hard text.

## License And Credit

License: CC-BY-4.0.

Credit: https://colinknapp.com
