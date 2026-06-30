#!/usr/bin/env bash
#
# Run once after cloning from the template.
# Substitutes PKGCRATE / PKGBIN placeholders and installs git hooks.
#
# Usage:
#   bash scripts/install.sh <crate-name> [binary-name]
#
# When crate name and binary name are the same (most projects):
#   bash scripts/install.sh mytool
#
# When they differ (e.g. crate published as "mytool-rust", binary is "mytool"):
#   bash scripts/install.sh mytool-rust mytool

set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: bash scripts/install.sh <crate-name> [binary-name]" >&2
  exit 1
fi

PKGCRATE="$1"
PKGBIN="${2:-$1}"

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

# --- substitute placeholders ---
files=$(grep -rl "PKGCRATE\|PKGBIN" . \
  --exclude-dir=.git \
  --exclude-dir=target \
  --exclude="$(basename "$0")")

for f in $files; do
  sed -i "s/PKGCRATE/${PKGCRATE}/g; s/PKGBIN/${PKGBIN}/g" "$f"
  echo "updated $f"
done

# --- install git hooks ---
hooks_dir="${repo_root}/scripts/hooks"
chmod +x "${hooks_dir}"/* 2>/dev/null || true
git -C "${repo_root}" config core.hooksPath scripts/hooks
echo "installed git hooks from scripts/hooks"

echo
echo "Done. Crate: ${PKGCRATE}, binary: ${PKGBIN}"
echo "Next steps:"
echo "  1. Fill in pkgdesc (and any extra deps) in ci.toml"
echo "  2. Add CI_DISPATCH_TOKEN to this repo's GitHub secrets"
echo "  3. bash scripts/setup-branch-protection.sh"
