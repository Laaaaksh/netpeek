#!/usr/bin/env bash
# Converts a raw screen recording of Netpeek into the README demo assets.
#
# Usage: scripts/record-demo/convert.sh [input.mov]
#   input.mov defaults to ~/Desktop/netpeek-demo.mov.
#
# Normally invoked by capture.sh, which produces that input.mov itself via
# window-mode screencapture - see scripts/record-demo/README.md.
set -euo pipefail

INPUT="${1:-$HOME/Desktop/netpeek-demo.mov}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="$REPO_ROOT/docs/assets"
MP4_OUT="$OUT_DIR/demo.mp4"
GIF_OUT="$OUT_DIR/demo.gif"
MAX_GIF_BYTES=$((10 * 1024 * 1024))

if [[ ! -f "$INPUT" ]]; then
  echo "error: input recording not found at $INPUT" >&2
  echo "Record the demo first - see scripts/record-demo/README.md." >&2
  exit 1
fi

mkdir -p "$OUT_DIR"

gif_size() {
  stat -f%z "$GIF_OUT" 2>/dev/null || stat -c%s "$GIF_OUT"
}

echo "==> Encoding MP4 (1280px wide, H.264, yuv420p) from $INPUT"
ffmpeg -y -i "$INPUT" \
  -vf "scale=1280:-2:flags=lanczos" \
  -c:v libx264 -pix_fmt yuv420p -movflags +faststart \
  -an \
  "$MP4_OUT"

WORKDIR="$(mktemp -d -t netpeek-demo)"
trap 'rm -rf "$WORKDIR"' EXIT
PALETTE="$WORKDIR/palette.png"

# Start at ~12fps and step down if the GIF comes out over the size budget,
# rather than guessing a single fps up front.
for FPS in 12 10 8 6; do
  echo "==> Encoding GIF (960px wide, ${FPS}fps) from $MP4_OUT"
  ffmpeg -y -i "$MP4_OUT" -vf "fps=${FPS},scale=960:-2:flags=lanczos,palettegen" "$PALETTE"
  ffmpeg -y -i "$MP4_OUT" -i "$PALETTE" \
    -filter_complex "fps=${FPS},scale=960:-2:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer" \
    "$GIF_OUT"

  SIZE="$(gif_size)"
  echo "    -> $GIF_OUT is $((SIZE / 1024 / 1024))MB at ${FPS}fps"
  if [[ "$SIZE" -le "$MAX_GIF_BYTES" ]]; then
    break
  fi
  echo "    over the 10MB budget, dropping fps and retrying..."
done

FINAL_SIZE="$(gif_size)"
if [[ "$FINAL_SIZE" -gt "$MAX_GIF_BYTES" ]]; then
  echo "error: $GIF_OUT is still $((FINAL_SIZE / 1024 / 1024))MB after dropping to 6fps." >&2
  echo "Shorten the source recording and re-run." >&2
  exit 1
fi

echo "==> Done."
echo "    $MP4_OUT"
echo "    $GIF_OUT ($((FINAL_SIZE / 1024 / 1024))MB)"
