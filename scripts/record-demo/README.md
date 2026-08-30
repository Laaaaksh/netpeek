# Recording the README demo

The conversion half of this is scripted (`convert.sh`); the capture half
isn't, and can't be.

## Why a human has to record it

Netpeek is a native macOS menu-bar app, not a localhost server, so the demo
has to be a real screen capture, not something generated headlessly. macOS
gates screen capture behind the Screen Recording TCC permission, granted
per top-level app. An automated/CI process (or an agent's shell) has no
practical way to get that permission granted to itself, and `screencapture`
/ `ffmpeg -f avfoundation` fail silently instead of erroring when it's
missing - `screencapture` writes a solid-black image at the right
dimensions, and avfoundation simply doesn't list a screen-capture device.
Both are easy to mistake for a working capture if you don't check.
So: a person with Screen Recording permission already granted to their
terminal/QuickTime records the ~50 seconds by hand, and everything after
that is this script.

## 1. Record it

1. Build and run the real app from this repo (not an installed copy):
   `npm run tauri dev`.
2. Give it real traffic to show - start a large download or a couple of
   video streams in the background so the per-process numbers actually move.
3. Press **⌘⇧5** → *Record Selected Portion* → drag a tight box around the
   menu-bar popover → Record. (QuickTime → File → New Screen Recording
   works the same way.)
4. Aim for 45-90 seconds, covering in order:
   - opening the menu-bar app
   - per-process bandwidth updating in real time - let it sit long enough
     that a viewer actually sees numbers change, since "live" is the claim
   - sorting to, or otherwise identifying, the heaviest app
   - the plain-English speed suggestion it produces for that app
   - anything on screen making the "no admin rights needed" point
5. Save it as `~/Desktop/netpeek-demo.mov`.

Close anything in the menu bar or process list you wouldn't want on a
public README before you start recording.

## 2. Convert it

```
scripts/record-demo/convert.sh [path-to-recording.mov]
```

(defaults to `~/Desktop/netpeek-demo.mov`), or `make demo` from the repo
root, which calls the same script.

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
