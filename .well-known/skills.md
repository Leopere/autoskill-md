# Skills

This file tells agents how to use autoskill-md.

## Purpose

This native CLI writes a skills file for agents.

## API

- Base path: `/`
- Source: No public HTTP routes found

## Auth

- No auth is needed to read this repo or run the CLI.
- Do not put secrets, tokens, or private keys in this file.

## Safe Actions

- Run through Rust, npm, Python, or Go wrappers.
- Scan local code comments and route hints.
- Write .well-known/skills.md.

## Risky Actions

- Ask before publishing files or changing a remote repo.
- Use --strict only when CI should fail.

## Limits

- Use a slow pace on large repos.
- Stop after repeated errors.

## More Info

- Docs: https://colinknapp.com/specs/skill-discovery.html
- Docs: https://github.com/Leopere/autoskill-md
- Docs: https://github.com/Leopere/autoskill-md#readme
- Support: https://github.com/Leopere/autoskill-md/issues

## Credits

- Spec: https://colinknapp.com/specs/skill-discovery.html
- Spec version: 2026-06-13
- Credit: https://colinknapp.com
- License: CC-BY-4.0
