#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RAW_DIR="$ROOT/assets/product-hunt/raw"
OUT_DIR="$ROOT/assets/product-hunt"
TEMPLATE_DIR="$ROOT/docs/product-hunt/templates"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/peep-ph-assets.XXXXXX")"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$RAW_DIR" "$OUT_DIR"

echo "[1/5] Exporting raw terminal snapshots"
cargo run --example export_marketing_assets -- --theme dark --output-dir "$RAW_DIR" >/dev/null
cargo run --example export_marketing_assets -- --theme light --output-dir "$RAW_DIR" >/dev/null

echo "[2/5] Capturing README screenshots"
playwright screenshot --viewport-size "1472,890" "file://$RAW_DIR/mock-overview-dark.svg" "$ROOT/assets/screenshot.png" >/dev/null
playwright screenshot --viewport-size "1472,890" "file://$RAW_DIR/mock-focus-dark.svg" "$ROOT/assets/screenshot-focus.png" >/dev/null
playwright screenshot --viewport-size "1472,890" "file://$RAW_DIR/empty-dark.svg" "$ROOT/assets/screenshot-empty.png" >/dev/null

echo "[3/5] Capturing Product Hunt gallery"
playwright screenshot --viewport-size "240,240" "file://$TEMPLATE_DIR/thumbnail.html" "$OUT_DIR/thumbnail.png" >/dev/null
playwright screenshot --viewport-size "1270,760" "file://$TEMPLATE_DIR/gallery-01-hero.html" "$OUT_DIR/gallery-01-hero.png" >/dev/null
playwright screenshot --viewport-size "1270,760" "file://$TEMPLATE_DIR/gallery-02-zero-config.html" "$OUT_DIR/gallery-02-zero-config.png" >/dev/null
playwright screenshot --viewport-size "1270,760" "file://$TEMPLATE_DIR/gallery-03-visibility.html" "$OUT_DIR/gallery-03-visibility.png" >/dev/null
playwright screenshot --viewport-size "1270,760" "file://$TEMPLATE_DIR/gallery-04-focus.html" "$OUT_DIR/gallery-04-focus.png" >/dev/null
playwright screenshot --viewport-size "1270,760" "file://$TEMPLATE_DIR/gallery-05-terminal.html" "$OUT_DIR/gallery-05-terminal.png" >/dev/null

echo "[4/5] Building demo video"
cat > "$TMP_DIR/slides.ffconcat" <<EOF
ffconcat version 1.0
file '$OUT_DIR/thumbnail.png'
duration 7
file '$OUT_DIR/gallery-01-hero.png'
duration 7
file '$OUT_DIR/gallery-02-zero-config.png'
duration 7
file '$OUT_DIR/gallery-03-visibility.png'
duration 7
file '$OUT_DIR/gallery-04-focus.png'
duration 7
file '$OUT_DIR/gallery-05-terminal.png'
duration 7
file '$OUT_DIR/gallery-05-terminal.png'
EOF

ffmpeg -y \
  -safe 0 \
  -i "$TMP_DIR/slides.ffconcat" \
  -vf "fps=30,scale=1270:760:force_original_aspect_ratio=decrease,pad=1270:760:(ow-iw)/2:(oh-ih)/2:color=0x0b1120,format=yuv420p" \
  -movflags +faststart \
  "$OUT_DIR/demo.mp4" >/dev/null 2>&1

cp "$OUT_DIR/gallery-01-hero.png" "$OUT_DIR/demo-poster.png"

echo "[5/5] Done"
ls -1 "$OUT_DIR" | sed 's/^/  - /'
