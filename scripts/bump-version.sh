#!/bin/bash
# Bump the crate patch version. Used when the combined spec hash changes.
# Usage: scripts/bump-version.sh
set -euo pipefail

CUR=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
if [[ ! "$CUR" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: version '$CUR' is not a plain MAJOR.MINOR.PATCH; refusing to bump" >&2
  exit 1
fi
IFS='.' read -r MAJOR MINOR PATCH <<< "$CUR"
NEW="${MAJOR}.${MINOR}.$((PATCH + 1))"

if [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i '' "s/^version = \".*\"/version = \"$NEW\"/" Cargo.toml
else
  sed -i "s/^version = \".*\"/version = \"$NEW\"/" Cargo.toml
fi

# Keep Cargo.lock's own package entry in sync. `cargo publish` refuses to run on a
# dirty tree; if the lock's autogen-stedi version lags Cargo.toml, publish regenerates
# it at release time, dirties the checkout, and aborts (this sank the v0.2.1 publish).
# Update only the version line inside the autogen-stedi package block, offline.
awk -v new="$NEW" '
  /^name = "autogen-stedi"$/ { inpkg = 1 }
  inpkg && /^version = / { sub(/"[^"]*"/, "\"" new "\""); inpkg = 0 }
  { print }
' Cargo.lock > Cargo.lock.tmp && mv Cargo.lock.tmp Cargo.lock

echo "$CUR -> $NEW"
