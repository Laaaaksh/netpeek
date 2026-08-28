# Netpeek

See which apps are eating your Mac's internet, live.

Netpeek is a small, open-source macOS menu-adjacent utility: a single window
showing which processes are currently using the network, how much bandwidth
each one is taking (download and upload, separately), and plain-English
suggestions for what's worth doing about it. Open it the moment your internet
feels slow.

> **Screenshot:** _coming soon - run `npm run tauri dev` to see it live in the
> meantime._

## What it does

- Live table of processes sorted by current speed, refreshed roughly once a
  second, with download and upload shown separately and a total-throughput
  header.
- A suggestions panel that explains, in plain language, things like:
  - which processes are the biggest bandwidth users right now
  - a background file-sync client (Dropbox, iCloud Drive, Google Drive,
    OneDrive) quietly syncing
  - a system or App Store update downloading in the background
  - Time Machine backing up over the network
  - a background process with no visible window sending/receiving data
    steadily
  - browser helper processes (Chrome's many renderer/GPU helpers, etc.)
    collapsed into a single row per browser, so the list stays readable
- Light and dark mode, following your system appearance (with a manual
  toggle).

## What it does NOT do

Netpeek is **read-only**. It observes and advises - it never quits,
suspends, throttles, or otherwise acts on any process. It needs **no admin
rights** (no `sudo`, no privileged helper, no permission prompt), and it
**sends no data anywhere**: the suggestion engine is a small rule-based
system that runs entirely offline, with no LLM, no API key, and no telemetry.
See [Verifying it makes no network calls](#verifying-it-makes-no-network-calls)
below for how to check that yourself.

## How it works

Netpeek shells out to macOS's built-in `/usr/bin/nettop` (the same tool
Activity Monitor's Network tab is built on) to read per-process byte
counters, without needing Full Disk Access, Accessibility, or any special
entitlement. Those counters are cumulative, so the Rust core keeps the
previous sample and computes `(current - previous) / elapsed_seconds` for
every process to get an actual per-second rate - never a raw cumulative
total. It also handles the edge cases that come with reading live system
counters: a process disappearing between samples, a pid being reused by an
unrelated process, a counter resetting, and the very first sample (which has
no prior sample to diff against, so it shows nothing rather than a huge
false spike).

## Build and run

Prerequisites:

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain, e.g. via
  `rustup` or `brew install rust`)
- Xcode Command Line Tools (`xcode-select --install`)

```bash
npm install
npm run tauri dev
```

This launches a window with a live process list. If you have something
downloading in the background (or just leave a browser tab open), you should
see non-zero download/upload speeds within a second or two.

To produce a distributable `.app` bundle:

```bash
npm run tauri build
```

The bundle is written under `src-tauri/target/release/bundle/macos/`.

### Gatekeeper note (unsigned build)

Prebuilt releases of Netpeek are ad-hoc signed (so they'll actually launch on
Apple Silicon) but are **not signed with a paid Apple Developer certificate
and are not notarized**. macOS Gatekeeper will refuse to open it the first
time. To run it:

1. Double-click the app. macOS will show a dialog saying it "cannot be
   opened" because Apple cannot verify it, with no bypass option in that
   dialog.
2. Open **System Settings > Privacy & Security**, scroll down to the
   Security section, and click **Open Anyway** next to the message about
   Netpeek.
3. Confirm in the prompt that follows. You only need to do this once per
   install.

(Right-click → Open no longer bypasses Gatekeeper for unsigned apps as of
recent macOS versions - the steps above are the current way through it.)

## Verifying it makes no network calls

Netpeek's only interaction with the outside world is spawning the local
`/usr/bin/nettop` process - the app itself never opens a socket. Ways to
confirm this yourself:

- **Read the code**: the Rust core (`src-tauri/src`) has no HTTP client, no
  networking crate, and no telemetry dependency in `Cargo.toml`. The only
  process it spawns is `/usr/bin/nettop` (see `src-tauri/src/nettop.rs`).
- **Little Snitch / Lulu / your firewall of choice**: run the app with a
  network-monitoring firewall active and confirm it never prompts for or
  makes an outbound connection.
- **`lsof`**: while the app is running, `lsof -a -p $(pgrep -f target/.*/netpeek) -i`
  will list any open sockets held by the process specifically (the `-a` is
  important - without it `lsof` OR's `-p` and `-i` together and shows every
  process on the machine with a socket open). There should be no output.

## Suggestion rules

The suggestions panel is a small, fully offline rule engine
(`src-tauri/src/suggestions.rs`) evaluated over the live process table on
every sample:

1. **Top bandwidth consumers** - ranks the biggest users by total throughput,
   using friendly app names where they're recognized.
2. **Background file sync** - flags Dropbox, iCloud Drive (`bird`), Google
   Drive, or OneDrive when they're actively syncing.
3. **System / App Store updates** - flags `softwareupdated` or
   `appstoreagent` downloading in the background.
4. **Time Machine** - flags `backupd` backing up over the network.
5. **Background chatter** - flags a process with no recognized application
   window that has sustained non-trivial traffic for a few consecutive
   samples.
6. **Browser helper aggregation** - Chrome/Safari/Firefox/Edge/Brave/Arc
   helper, renderer, and GPU processes are merged into a single row under
   their parent browser; when that merge is actually collapsing more than
   one process, a note says so.

## Project layout

- `src/` - React + TypeScript UI (Vite, Tailwind CSS, shadcn/ui).
- `src-tauri/` - Rust core: nettop process management (`nettop.rs`),
  rate calculation (`sampler.rs`), friendly-name/category mapping
  (`catalog.rs`), and the suggestion engine (`suggestions.rs`).

## License

MIT - see [LICENSE](LICENSE).
