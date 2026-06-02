#!/usr/bin/env bash
# Remove all node_modules/ and target/ directories under the repo root.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DRY_RUN=false
COUNT=0


usage() {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Delete every node_modules and target folder under:
  $ROOT

Options:
  -n, --dry-run   List what would be removed without deleting
  -h, --help      Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -n | --dry-run) DRY_RUN=true; shift ;;
    -h | --help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage >&2; exit 1 ;;
  esac
done

while IFS= read -r -d '' dir; do
  COUNT=$((COUNT + 1))
  if [[ "$DRY_RUN" == true ]]; then
    echo "would remove: $dir"
  else
    rm -rf "$dir"
    echo "removed: $dir"
  fi
done < <(
  find "$ROOT" \
    \( -path "$ROOT/.git" -o -path "$ROOT/.git/*" \) -prune -o \
    -type d \( -name node_modules -o -name target \) -print0
)

if [[ "$COUNT" -eq 0 ]]; then
  echo "Nothing to clean."
else
  echo "Done ($COUNT directories)."
fi
