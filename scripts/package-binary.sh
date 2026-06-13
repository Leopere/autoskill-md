#!/usr/bin/env bash
set -euo pipefail

target="${TARGET:-$(rustc -vV | awk '/host:/ {print $2}')}"
exe="autoskill-md"
if [[ "$target" == *"windows"* ]]; then
  exe="autoskill-md.exe"
fi

if [ -n "${TARGET:-}" ]; then
  cargo build --release --target "$target"
else
  cargo build --release
fi

build_dir="target/release"
if [ -n "${TARGET:-}" ]; then
  build_dir="target/$target/release"
fi

rm -rf dist/package
mkdir -p dist/package
cp "$build_dir/$exe" "dist/package/$exe"

mkdir -p dist
tar -czf "dist/autoskill-md-$target.tar.gz" -C dist/package "$exe"

echo "dist/autoskill-md-$target.tar.gz"
