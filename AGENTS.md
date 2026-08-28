# Project agent memory

This file is the project's committed home for project-intrinsic agent knowledge: build, test, release, architecture, and sharp-edge notes that should travel with the code.

- Requires a Rust toolchain (`cargo`/`rustc`) in addition to Node - not preinstalled on a fresh macOS setup; `brew install rust` works.
- Frontend: `npm run build` (tsc + vite). Rust: `cargo build` / `cargo test` from `src-tauri/`. Full app: `npm run tauri dev` / `npm run tauri build`.
- `nettop` fully buffers its stdout when piped (updates arrive in multi-second bursts instead of ~1/sec) - it must be spawned under a PTY to get timely per-second flushes. See the module doc in `src-tauri/src/nettop.rs`.
- Product scope is firmly read-only and offline: no process control, no admin/`sudo`, no network calls of the app's own (verified via dependency tree + `lsof`, see README's "Verifying it makes no network calls"). Don't add either without checking with the maintainer first - both are explicit product decisions, not oversights.
- Rate math, catalog/aggregation rules, and the suggestion engine each have unit tests colocated in their module (`sampler.rs`, `catalog.rs`, `suggestions.rs`) - extend those when changing behavior there rather than only testing by hand.

## Maintaining this file

Keep this file for knowledge useful to almost every future agent session in this project.
Do not repeat what the codebase already shows; point to the authoritative file or command instead.
Prefer rewriting or pruning existing entries over appending new ones.
When updating this file, preserve this bar for all agents and keep entries concise.
