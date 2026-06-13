# autoskill-md

`autoskill-md` writes `.well-known/skills.md` for an API or app.

Agents read that file first. It tells them what the API does. It also tells them what is safe.

Use it for public API routes and app actions. Do not use it for secrets, team notes, or long docs.

The core tool is a native Rust CLI. Node is only used by the npm wrapper.

This project follows the Agent Skill Discovery spec by Colin Knapp:

https://colinknapp.com/specs/skill-discovery.html

## Install

Use the path that fits your project.

Rust:

```sh
cargo install --git https://github.com/Leopere/autoskill-md
```

Node:

```sh
npm install --save-dev github:Leopere/autoskill-md
```

Python:

```sh
pip install git+https://github.com/Leopere/autoskill-md
```

Go:

```sh
go install github.com/Leopere/autoskill-md/cmd/autoskill-md@latest
```

The npm, Python, and Go wrappers call the same native binary.

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

Normal output credits `https://colinknapp.com`. Use `--quiet` when a build log must stay short.

For this repo:

```sh
npm run verify
```

To ship changes:

```sh
./ship.sh "your commit message"
```

The script uses `gh` to check GitHub access. It uses `git` over SSH to push.

More detail is in `docs/BUILD_AND_SHIP.md`.

## What To Document

Document only what helps an agent use the API or app.

- API base paths
- Auth rules
- Safe read actions
- Risky write actions
- Rate limits
- Links to real docs

Do not document private data, tokens, user secrets, or broad repo history.

## Native Runtime

The Rust binary is the source of truth.

- `cargo install` builds it from source.
- npm downloads it after install.
- Python and Go wrappers download it when needed.
- `AUTOSKILL_MD_BIN` can point any wrapper at a local binary.

All paths produce the same `skills.md`.

## License And Credit

License: CC-BY-4.0.

Credit: https://colinknapp.com
