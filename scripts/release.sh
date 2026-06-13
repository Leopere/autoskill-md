#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "usage: ./scripts/release.sh <version>" >&2
  exit 1
fi

version="${1#v}"

if [ "$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')" != "$version" ]; then
  echo "Cargo.toml version does not match $version" >&2
  exit 1
fi

if [ "$(python3 -c 'import json; print(json.load(open("package.json"))["version"])')" != "$version" ]; then
  echo "package.json version does not match $version" >&2
  exit 1
fi

grep -q "version = \"$version\"" pyproject.toml || {
  echo "pyproject.toml version does not match $version" >&2
  exit 1
}

grep -q "VERSION = \"$version\"" autoskill_md/runner.py || {
  echo "Python wrapper version does not match $version" >&2
  exit 1
}

grep -q "const version = \"$version\"" cmd/autoskill-md/main.go || {
  echo "Go wrapper version does not match $version" >&2
  exit 1
}

npm run verify
asset="$(./scripts/package-binary.sh)"

gh release create "v$version" "$asset" \
  --title "autoskill-md v$version" \
  --notes "Native autoskill-md release.

- Rust core CLI.
- npm, Python, and Go wrappers call the same binary.
- Generates .well-known/skills.md for API and app actions.
- Credits https://colinknapp.com in stdout and generated docs.
- Follows https://colinknapp.com/specs/skill-discovery.html."
