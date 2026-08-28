# Project agent memory

This file is the project's committed home for project-intrinsic agent knowledge: build, test, release, architecture, and sharp-edge notes that should travel with the code.

- Requires a Rust toolchain (`cargo`/`rustc`) in addition to Node - not preinstalled on a fresh macOS setup; `brew install rust` works.
- Frontend: `npm run build` (tsc + vite). Rust: `cargo build` / `cargo test` from `src-tauri/`. Full app: `npm run tauri dev` / `npm run tauri build`.
- `nettop` fully buffers its stdout when piped (updates arrive in multi-second bursts instead of ~1/sec) - it must be spawned under a PTY to get timely per-second flushes. See the module doc in `src-tauri/src/nettop.rs`.
- Product scope is firmly read-only and offline: no process control, no admin/`sudo`, no network calls of the app's own (verified via dependency tree + `lsof`, see README's "Verifying it makes no network calls"). Don't add either without checking with the maintainer first - both are explicit product decisions, not oversights.
- Rate math, catalog/aggregation rules, and the suggestion engine each have unit tests colocated in their module (`sampler.rs`, `catalog.rs`, `suggestions.rs`) - extend those when changing behavior there rather than only testing by hand.
- Every catalog entry (`catalog.rs`) carries a plain-English `what_it_is` sentence and an actionable `verdict`, not just a display name - an unrecognized process gets a fixed "not recognized" explanation rather than an invented one. Chromium-family browsers (Chrome/Edge/Brave/Arc/Opera) and Safari also carry a `breakdown_kind`; `procinfo.rs` reads helper processes' full command lines via `ps` and `breakdown.rs` classifies them (page content/graphics/supporting service/extension) from argv flags (`--type=`, `--utility-sub-type=`, `--extension-process`). `ps`, like `nettop`, is a local read-only spawn - it doesn't affect the no-network-calls property.
- Confirmed live against a running multi-tab Chrome with an extension loaded: modern Chrome funnels almost all actual socket I/O - tabs and extensions alike - through its single shared Network Service utility process rather than the renderer that requested it, so `nettop`/`ps` (and this app's breakdown) will usually attribute most of a Chrome row's traffic to "Network connections" even when a specific tab or extension is driving it. That's a real limit of what's observable from outside the browser, not a bug - it's also why the Chromium browsers' expanded row hands off to the browser's own Task Manager for exact per-tab/extension attribution.

## Maintaining this file

Keep this file for knowledge useful to almost every future agent session in this project.
Do not repeat what the codebase already shows; point to the authoritative file or command instead.
Prefer rewriting or pruning existing entries over appending new ones.
When updating this file, preserve this bar for all agents and keep entries concise.
