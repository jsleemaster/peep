#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/peep-launch-check.XXXXXX")"
trap 'rm -rf "$TMP_DIR"' EXIT

cd "$ROOT"

echo "[1/4] Running tests"
cargo test >/dev/null

echo "[2/4] Verifying cargo install"
CARGO_NET_OFFLINE=true cargo install --path "$ROOT" --root "$TMP_DIR/cargo" --locked --offline --force >/dev/null
"$TMP_DIR/cargo/bin/peep" --help >/dev/null

echo "[3/4] Verifying release tarball layout"
CARGO_NET_OFFLINE=true cargo build --release --offline >/dev/null

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64)
    ASSET_NAME="peep-macos-arm64.tar.gz"
    ;;
  Darwin-x86_64)
    ASSET_NAME="peep-macos-intel.tar.gz"
    ;;
  Linux-x86_64)
    ASSET_NAME="peep-linux-x86_64.tar.gz"
    ;;
  Linux-aarch64)
    ASSET_NAME="peep-linux-arm64.tar.gz"
    ;;
  *)
    echo "Unsupported platform for asset-name verification: $(uname -s)-$(uname -m)"
    exit 1
    ;;
esac

mkdir -p "$TMP_DIR/release"
tar czf "$TMP_DIR/$ASSET_NAME" -C "$ROOT/target/release" peep
tar xzf "$TMP_DIR/$ASSET_NAME" -C "$TMP_DIR/release"
"$TMP_DIR/release/peep" --help >/dev/null

echo "[4/4] Optional Homebrew verification"
if [[ "${VERIFY_BREW:-0}" == "1" ]]; then
  brew info jsleemaster/tap/peep >/dev/null
  echo "Homebrew formula is reachable."
else
  echo "Skipped. Re-run with VERIFY_BREW=1 after the release is published."
fi

echo "Launch surface checks passed."
