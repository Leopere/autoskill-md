# Build And Ship

This repo should stay small and clear.

## Goal

`autoskill-md` helps apps publish a short skills file.

That file is for agents. It should explain API routes, app actions, auth, safe calls, risky calls, limits, and links to docs.

It should not hold secrets. It should not replace full docs.

The core tool is a Rust binary.

Wrappers for npm, Python, and Go must call that same binary.

## Build Rules

Use these scripts:

```sh
npm run build
npm run check
npm test
npm run test:wrappers
npm run verify
```

`npm run build` builds the Rust binary and writes `.well-known/skills.md`.

`npm run check` checks that the file is fresh, safe, and easy to read.

`npm run verify` runs the full pre-ship path.

## Writing Rules

Keep text below grade 7.

Use short words.

Use short lines.

Tell agents what they may do.

Tell agents when to ask first.

Do not add secrets, tokens, keys, passwords, or private user data.

## Shipping

Use GitHub CLI and Git over SSH.

```sh
gh auth status
git remote -v
./ship.sh "your commit message"
```

`ship.sh` runs `npm run verify`.

Then it stages files, commits them, and pushes the current branch to GitHub.

For a release:

```sh
./scripts/release.sh 0.2.1
```

Tagged GitHub releases build native binary files for npm, Python, and Go wrappers.
