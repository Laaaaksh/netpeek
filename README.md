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

## Download and install

1. Go to the [Releases page](https://github.com/Laaaaksh/netpeek/releases)
   and download the latest `Netpeek-*-macos-universal.zip`. It's a universal
   binary that runs natively on both Apple Silicon and Intel Macs.
2. Unzip it (double-click the file if Finder doesn't do it for you).
3. Drag `Netpeek.app` into your `Applications` folder.
4. Open Netpeek from Applications (or Spotlight/Launchpad).

The first time you open it, macOS will refuse and show a dialog saying
Netpeek "cannot be opened" or is "Not Opened" because Apple cannot verify
it. This is expected - Netpeek isn't signed with a paid Apple Developer
certificate (that costs $99/year, and the maintainer hasn't set one up), so
Gatekeeper doesn't recognize it. The app itself is unaffected; nothing about
it is broken. To open it anyway:

1. Dismiss the "Not Opened" dialog.
2. Open **System Settings > Privacy & Security**, and scroll down to the
   Security section near the bottom.
3. You'll see a message about Netpeek being blocked - click **Open Anyway**
   next to it.
4. Confirm in the prompt that follows (you may need to enter your Mac
   password or use Touch ID).

You only need to do this once per install. After that, Netpeek opens
normally like any other app.

(There is no right-click → Open workaround on current macOS versions -
Apple removed it. System Settings is the only way through.)

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

A build you produce locally is ad-hoc signed, same as the release builds, so
it hits the same Gatekeeper warning on first launch. See
[Download and install](#download-and-install) above for the "Open Anyway"
steps.

## Verifying it makes no network calls

Netpeek's only interactions with the outside world are spawning two local,
read-only system binaries - the app itself never opens a socket:

- `/usr/bin/nettop`, for per-process byte counters (`src-tauri/src/nettop.rs`).
- `/bin/ps`, to read a process's command line so a Chromium-family browser's
  helper processes can be classified as page content, graphics, a supporting
  service, or an extension (`src-tauri/src/procinfo.rs`). This only reads the
  local process table; it does not touch the network either.

Ways to confirm this yourself:

- **Read the code**: the Rust core (`src-tauri/src`) has no HTTP client, no
  networking crate, and no telemetry dependency in `Cargo.toml` (unchanged
  by the `ps`-based breakdown - it's a `std::process::Command` call, not a
  new dependency).
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
6. **Browser helper aggregation** - Chrome/Safari/Firefox/Edge/Brave/Arc/Opera
   helper, renderer, and GPU processes are merged into a single row under
   their parent browser; when that merge is actually collapsing more than
   one process, a note says so.

## Explaining processes, and breaking browsers open

Every entry in `src-tauri/src/catalog.rs` carries a plain-English sentence
saying what the process is and a verdict saying whether the user needs to do
anything - a process Netpeek doesn't recognize is labelled as such rather
than given an invented explanation. The UI shows both when a process row is
expanded.

Chromium-family browsers (Chrome, Edge, Brave, Arc, Opera) get a further,
real breakdown of that expanded row: `procinfo.rs` reads each helper
process's full command line via `ps`, and `breakdown.rs` classifies it from
its `--type=`/`--utility-sub-type=`/`--extension-process` flags into page
content, graphics, a supporting service, or a browser extension. Safari's
helpers are already named distinctly by process name
(`com.apple.WebKit.WebContent`/`.Networking`/`.GPU`), so `breakdown.rs`
classifies those without needing argv. Firefox stays aggregated - its
multiprocess model doesn't expose a stable, name- or argv-based way to tell
its helpers apart the way Chromium's does.

Per-tab attribution is not available to an outside app on stable Chrome (the
API that could do it is Dev-channel only, and the alternative requires
relaunching Chrome with a remote debugging port, which Netpeek deliberately
does not do or suggest). When a Chromium-family browser is a significant
consumer, its expanded row instead points the user at that browser's own
Task Manager, which can name the exact tab or extension.

One thing to expect when trying this live: modern Chrome routes nearly all
actual socket I/O - for tabs and extensions alike - through its single
shared Network Service utility process rather than the renderer that
requested it, so most of a Chrome row's traffic will usually land in a
"Network connections" bucket rather than "Page content" or "Browser
extension" even though the classification logic is correct. This is a limit
of what `nettop`/`ps` can see from outside the browser, not a bug - see the
module doc on `breakdown.rs`.

## Project layout

- `src/` - React + TypeScript UI (Vite, Tailwind CSS, shadcn/ui).
- `src-tauri/` - Rust core: nettop process management (`nettop.rs`),
  command-line lookup for browser helpers (`procinfo.rs`), rate calculation
  (`sampler.rs`), friendly-name/category/explanation mapping (`catalog.rs`),
  Chromium/Safari helper classification (`breakdown.rs`), and the suggestion
  engine (`suggestions.rs`).

## License

MIT - see [LICENSE](LICENSE).
