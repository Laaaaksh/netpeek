mod breakdown;
mod catalog;
mod nettop;
mod procinfo;
mod sampler;
mod suggestions;

use sampler::{Sampler, Snapshot};
use serde::Serialize;
use std::sync::mpsc;
use std::sync::Mutex;
use suggestions::Suggestion;
use tauri::{Emitter, Manager};

const BANDWIDTH_EVENT: &str = "bandwidth-update";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotPayload {
    snapshot: Snapshot,
    suggestions: Vec<Suggestion>,
}

#[derive(Default)]
struct AppState {
    latest: Mutex<Option<SnapshotPayload>>,
}

/// Returns the most recent snapshot so the UI has something to render
/// immediately on launch, before the next `bandwidth-update` event arrives.
#[tauri::command]
fn get_latest_snapshot(state: tauri::State<AppState>) -> Option<SnapshotPayload> {
    state.latest.lock().unwrap().clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![get_latest_snapshot])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let (tx, rx) = mpsc::channel();
            nettop::spawn_reader(tx);

            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut sampler = Sampler::new();
                while let Ok(raw_batch) = rx.recv() {
                    let argv_by_pid = procinfo::snapshot_argv();
                    let Some(snapshot) = sampler.ingest(raw_batch, &argv_by_pid) else {
                        continue;
                    };
                    let suggestions = suggestions::build(&snapshot.processes);
                    if let Some(top) = snapshot.processes.first() {
                        log::info!(
                            "sample: {} processes, top={} down={:.0}B/s up={:.0}B/s, {} suggestions",
                            snapshot.processes.len(),
                            top.display_name,
                            top.download_bps,
                            top.upload_bps,
                            suggestions.len()
                        );
                    }
                    let payload = SnapshotPayload {
                        snapshot,
                        suggestions,
                    };

                    if let Some(state) = app_handle.try_state::<AppState>() {
                        *state.latest.lock().unwrap() = Some(payload.clone());
                    }
                    let _ = app_handle.emit(BANDWIDTH_EVENT, &payload);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
