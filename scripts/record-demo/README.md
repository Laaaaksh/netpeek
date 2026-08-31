# Recording the README demo

`capture.sh` records Netpeek's own window and hands off to `convert.sh`.

## Why window-mode capture

Netpeek is a native macOS window (Tauri v2 + React), not a localhost server,
so the demo has to be a real screen capture. Early attempts captured a
region of the screen and picked up whatever else was on it - other windows,
notifications, browser tabs. `screencapture -l<windowid>` avoids that
entirely: it records only the named window's own pixels against a solid
black backdrop, so nothing else on the desktop can appear in frame. That
matters because this recording ends up in a public README.

The Accessibility API (`System Events`) can move and resize a window but
doesn't expose its `CGWindowID`, so `window_id.swift` gets it directly from
CoreGraphics (`CGWindowListCopyWindowInfo`).

This needs the macOS Screen Recording permission granted to the calling
terminal (System Settings > Privacy & Security > Screen Recording).
`screencapture`/`ffmpeg -f avfoundation` fail silently instead of erroring
when it's missing - `screencapture` writes a solid-black image at the right
dimensions, and avfoundation simply doesn't list a screen-capture device.
Both are easy to mistake for a working capture if you don't check.

## 1. Give it real traffic

Start something bandwidth-heavy in another terminal before recording, so
the per-process numbers actually move - e.g.:

```
curl -o /dev/null http://speedtest.tele2.net/1GB.zip
```

## 2. Record and convert

```
scripts/record-demo/capture.sh [seconds]
```

(defaults to 65s), or `make demo` from the repo root. This builds the
release app if needed, launches it, finds its window, records it, then
calls `convert.sh` and cleans up the raw capture. Live per-process
bandwidth updating in real time and the plain-English suggestions panel
(which identifies the heaviest bandwidth user) are visible from the moment
the window opens - no extra clicking needed to sort or drill in.

To convert a recording you already have (e.g. one made by hand with
QuickTime), skip straight to:

```
scripts/record-demo/convert.sh path/to/recording.mov
```

This produces:

- `docs/assets/demo.mp4` - 1280px wide, H.264, `yuv420p`, for the "full
  quality" link in the README.
- `docs/assets/demo.gif` - 960px wide, starting at 12fps, built via
  ffmpeg's `palettegen`/`paletteuse` for a clean palette. If it comes out
  over the 10MB budget the script automatically steps the frame rate down
  (12 → 10 → 8 → 6fps) and retries; if it's still too big at 6fps, shorten
  the source recording rather than compressing further.

## 3. Verify before committing

```
ffprobe docs/assets/demo.gif
ffprobe docs/assets/demo.mp4
ls -lh docs/assets/demo.gif docs/assets/demo.mp4
```

Open the GIF and actually watch it play the loop - a handful of frames or a
static first frame is a failed capture, not a usable demo. Check the frame
count/duration and dimensions match what you recorded, and that the GIF is
under 10MB.

Even with window-mode capture eliminating everything outside the app,
step through a few frames of the raw recording before converting - a
tooltip, an autocomplete dropdown, or a file path rendered inside
Netpeek's own UI could still be worth a second look on a public README.
