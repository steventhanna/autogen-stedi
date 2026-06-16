#!/bin/bash
set -euo pipefail

# Regenerate every Stedi service module from the upstream OpenAPI specs.
#
# Stedi ships one spec per API, each on its own host. We generate each into a temp dir, then
# vendor only its `src/apis` and `src/models` into `src/<service>/`, rewriting the generator's
# absolute `crate::apis` / `crate::models` paths to `crate::<service>::…` so the code compiles
# inside a submodule. The run is idempotent: an unchanged spec reproduces the committed tree.
#
# Requires: openapi-generator (brew install openapi-generator) and a JDK.
# In CI set OPENAPI_GENERATOR=openapi-generator-cli.

# Portable in-place sed (macOS uses -i '', Linux uses -i).
sedi() {
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "$@"
  else
    sed -i "$@"
  fi
}

SPEC_BASE="https://raw.githubusercontent.com/Stedi/openapi/main"
# Feature name == rust module name, except '-' becomes '_' for the module.
SERVICES="claims core enrollment event-destinations healthcare manager payers"
GEN="${OPENAPI_GENERATOR:-openapi-generator}"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "==> Fetching Stedi OpenAPI specs from $SPEC_BASE ..."
for svc in $SERVICES; do
  curl -sS -L -o "$WORK/$svc.json" "$SPEC_BASE/$svc.json"
  echo "    $svc.json: $(wc -c < "$WORK/$svc.json" | tr -d ' ') bytes"
done

echo "==> Recording combined spec hash..."
( cd "$WORK" && shasum -a 256 $(for s in $SERVICES; do echo "$s.json"; done) \
  | sort | shasum -a 256 | awk '{print $1}' ) > SPEC_HASH
echo "    SPEC_HASH = $(cat SPEC_HASH)"

for svc in $SERVICES; do
  module="${svc//-/_}"
  echo "==> Generating '$svc' -> src/$module ..."

  rm -rf "$WORK/gen-$svc"
  "$GEN" generate \
    -i "$WORK/$svc.json" \
    -g rust \
    --library reqwest \
    --skip-validate-spec \
    --additional-properties=packageName=autogen-stedi-$svc,supportAsync=true \
    -o "$WORK/gen-$svc" \
    2>&1 | tail -3

  rm -rf "src/$module"
  mkdir -p "src/$module"
  cp -R "$WORK/gen-$svc/src/apis" "src/$module/apis"
  cp -R "$WORK/gen-$svc/src/models" "src/$module/models"

  # Rewrite absolute crate paths so the vendored code resolves inside src/$module/.
  # Order matters: the grouped-import form `crate::{apis::…, models}` is rewritten first; the
  # standalone `crate::apis` / `crate::models` forms are then caught without double-rewriting.
  while IFS= read -r -d '' f; do
    sedi \
      -e "s/crate::{/crate::$module::{/g" \
      -e "s/crate::apis/crate::$module::apis/g" \
      -e "s/crate::models/crate::$module::models/g" \
      "$f"
  done < <(find "src/$module" -name '*.rs' -print0)

  printf 'pub mod apis;\npub mod models;\n' > "src/$module/mod.rs"
done

echo "==> Applying post-generation fixes..."
# Some Stedi specs declare two properties that differ only in case (e.g. payerID / payerId);
# the rust generator snake_cases both to one field, producing an uncompilable duplicate. This
# idempotent pass renames the earlier (deprecated) collision to <name>_legacy<n>.
python3 scripts/fix-duplicate-fields.py src

echo "==> Verifying compilation..."
cargo check --all-features
cargo check --no-default-features --features "claims,native-tls"

echo "==> Done. Review changes with: git diff"
