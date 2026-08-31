#!/usr/bin/env bash
# Records a genuine screen capture of Netpeek's own window for the README
# demo, then hands off to convert.sh.
#
# Usage: scripts/record-demo/capture.sh [seconds]
#   seconds defaults to 65.
#
# Requires the calling terminal to already have the macOS Screen Recording
# permission (System Settings > Privacy & Security > Screen Recording).
#
# Why window-mode capture: `screencapture -l<windowid>` records only the
# named window's own pixels against a solid black backdrop - nothing else on
# the desktop (other apps, the Dock, notifications) can appear in frame,
# which matters because this recording ends up in a public README.
set -euo pipefail

DURATION="${1:-65}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_PATH="$REPO_ROOT/src-tauri/target/release/bundle/macos/Netpeek.app"
OUT="$REPO_ROOT/docs/assets/.demo-raw.mov"

if [[ ! -d "$APP_PATH" ]]; then
  echo "==> Building release app first: npm run tauri build"
  (cd "$REPO_ROOT" && npm run tauri build)
fi

echo "==> Give Netpeek real traffic to show before recording - e.g. in another"
echo "    terminal: curl -o /dev/null http://speedtest.tele2.net/1GB.zip"
echo

pkill -f "$APP_PATH/Contents/MacOS/netpeek" 2>/dev/null || true
sleep 1
open -a "$APP_PATH"
sleep 1.5
osascript -e 'tell application "System Events" to tell process "netpeek" to set frontmost to true'

WINDOW_ID="$(swift "$(dirname "${BASH_SOURCE[0]}")/window_id.swift" netpeek | sed -n 's/^id=\([0-9]*\).*/\1/p' | head -1)"
if [[ -z "$WINDOW_ID" ]]; then
  echo "error: could not find Netpeek's window id - is the app running and its window open?" >&2
  exit 1
fi
echo "==> Recording window id $WINDOW_ID for ${DURATION}s"

screencapture -v -V"$DURATION" -l"$WINDOW_ID" -x "$OUT"

echo "==> Raw capture at $OUT"
echo "==> Converting..."
"$(dirname "${BASH_SOURCE[0]}")/convert.sh" "$OUT"
rm -f "$OUT"
