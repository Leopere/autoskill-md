#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "usage: ./ship.sh <commit message>" >&2
  exit 1
fi

message="$*"
current_branch="$(git rev-parse --abbrev-ref HEAD)"
origin_url="$(git remote get-url origin)"

case "$origin_url" in
  git@github.com:*|ssh://git@github.com/*) ;;
  *)
    echo "origin must be a GitHub SSH remote." >&2
    echo "current origin: $origin_url" >&2
    exit 1
    ;;
esac

gh auth status >/dev/null
repo_url="$(gh repo view --json url -q .url)"
echo "GitHub repo: $repo_url"

npm run verify

git add .

if git diff --cached --quiet; then
  echo "No changes to ship."
  exit 0
fi

git commit -m "$message"
git push -u origin "$current_branch"

echo "Shipped $current_branch to GitHub."
