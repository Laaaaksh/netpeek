//! Rule-based, fully offline suggestion engine. No network calls, no LLM -
//! every suggestion is a plain-language rule evaluated over the live
//! process table produced by the sampler.

use crate::catalog::{Category, NOTABLE_BPS};
use crate::sampler::ProcessRate;
use serde::Serialize;

/// Threshold for calling something a "top consumer".
const TOP_CONSUMER_BPS: f64 = 50.0 * 1024.0;
/// How many consecutive active samples counts as "sustained" background chatter.
const SUSTAINED_SAMPLES: u32 = 3;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SuggestionKind {
    TopConsumer,
    Sync,
    Update,
    Backup,
    Background,
    Info,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    pub id: String,
    pub kind: SuggestionKind,
    pub title: String,
    pub detail: String,
}

pub fn build(processes: &[ProcessRate]) -> Vec<Suggestion> {
    let mut out = Vec::new();
    out.extend(top_consumers(processes));
    out.extend(sync_activity(processes));
    out.extend(update_activity(processes));
    out.extend(backup_activity(processes));
    out.extend(background_chatter(processes));
    out.extend(browser_aggregation_notes(processes));
    out
}

/// Rule 1: rank the top bandwidth consumers using friendly names.
fn top_consumers(processes: &[ProcessRate]) -> Vec<Suggestion> {
    processes
        .iter()
        .filter(|p| total_bps(p) >= TOP_CONSUMER_BPS)
        .take(3)
        .enumerate()
        .map(|(rank, p)| Suggestion {
            id: format!("top-consumer-{}", p.group_key),
            kind: SuggestionKind::TopConsumer,
            title: format!("#{} bandwidth user: {}", rank + 1, p.display_name),
            detail: format!(
                "{} is using {} right now ({} down, {} up).",
                p.display_name,
                format_bps(total_bps(p)),
                format_bps(p.download_bps),
                format_bps(p.upload_bps)
            ),
        })
        .collect()
}

/// Rule 2: background file sync clients (Dropbox, iCloud Drive, Google Drive, OneDrive).
fn sync_activity(processes: &[ProcessRate]) -> Vec<Suggestion> {
    by_category(processes, Category::Sync)
        .map(|p| Suggestion {
            id: format!("sync-{}", p.group_key),
            kind: SuggestionKind::Sync,
            title: format!("{} is syncing in the background", p.display_name),
            detail: format!(
                "{} is using {} to sync files. If your internet feels slow, pausing it until you need it will free that bandwidth up immediately.",
                p.display_name,
                format_bps(total_bps(p))
            ),
        })
        .collect()
}

/// Rule 3: system or App Store updates downloading.
fn update_activity(processes: &[ProcessRate]) -> Vec<Suggestion> {
    by_category(processes, Category::Update)
        .map(|p| Suggestion {
            id: format!("update-{}", p.group_key),
            kind: SuggestionKind::Update,
            title: format!("{} is downloading", p.display_name),
            detail: format!(
                "{} is using {} in the background. This is probably why things feel slow - it'll speed back up once the download finishes.",
                p.display_name,
                format_bps(total_bps(p))
            ),
        })
        .collect()
}

/// Rule 4: Time Machine backing up over the network.
fn backup_activity(processes: &[ProcessRate]) -> Vec<Suggestion> {
    by_category(processes, Category::Backup)
        .map(|p| Suggestion {
            id: format!("backup-{}", p.group_key),
            kind: SuggestionKind::Backup,
            title: "Time Machine is backing up".to_string(),
            detail: format!(
                "Time Machine is using {} to back up over the network. You can pause it from Time Machine settings if you need the bandwidth right now.",
                format_bps(total_bps(p))
            ),
        })
        .collect()
}

/// Rule 5: sustained traffic from a process with no visible window.
fn background_chatter(processes: &[ProcessRate]) -> Vec<Suggestion> {
    processes
        .iter()
        .filter(|p| {
            p.category == Category::Unrecognized
                && total_bps(p) >= NOTABLE_BPS
                && p.sustained_seconds >= SUSTAINED_SAMPLES
        })
        .take(2)
        .map(|p| Suggestion {
            id: format!("background-{}", p.group_key),
            kind: SuggestionKind::Background,
            title: format!("{} is chattering in the background", p.display_name),
            detail: format!(
                "\"{}\" has no visible window but has been steadily using {} for at least {} seconds. If you don't recognize it, it's worth checking what it is.",
                p.display_name,
                format_bps(total_bps(p)),
                p.sustained_seconds
            ),
        })
        .collect()
}

/// Rule 6: browser helper processes are aggregated under their parent
/// browser (in the sampler). Surface that as a visible, verifiable fact
/// whenever it's actually collapsing more than one process.
fn browser_aggregation_notes(processes: &[ProcessRate]) -> Vec<Suggestion> {
    processes
        .iter()
        .filter(|p| p.category == Category::App && p.pid_count > 1)
        .map(|p| Suggestion {
            id: format!("aggregation-{}", p.group_key),
            kind: SuggestionKind::Info,
            title: format!("{} helper processes combined", p.display_name),
            detail: format!(
                "{} runs as {} separate processes; they're shown as a single row here so the list stays readable.",
                p.display_name, p.pid_count
            ),
        })
        .collect()
}

fn by_category(processes: &[ProcessRate], category: Category) -> impl Iterator<Item = &ProcessRate> {
    processes
        .iter()
        .filter(move |p| p.category == category && total_bps(p) >= NOTABLE_BPS)
}

fn total_bps(p: &ProcessRate) -> f64 {
    p.download_bps + p.upload_bps
}

fn format_bps(bps: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    if bps >= GB {
        format!("{:.1} GB/s", bps / GB)
    } else if bps >= MB {
        format!("{:.1} MB/s", bps / MB)
    } else if bps >= KB {
        format!("{:.0} KB/s", bps / KB)
    } else {
        format!("{:.0} B/s", bps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(display_name: &str, category: Category, down: f64, up: f64, sustained: u32, pid_count: usize) -> ProcessRate {
        ProcessRate {
            group_key: display_name.to_lowercase().replace(' ', "-"),
            display_name: display_name.to_string(),
            category,
            download_bps: down,
            upload_bps: up,
            pid_count,
            sustained_seconds: sustained,
            what_it_is: "test fixture".to_string(),
            verdict: "test fixture".to_string(),
            breakdown: None,
            task_manager_hint: None,
        }
    }

    #[test]
    fn flags_dropbox_sync() {
        let processes = vec![process("Dropbox (file sync)", Category::Sync, 500_000.0, 10_000.0, 5, 1)];
        let suggestions = build(&processes);
        assert!(suggestions.iter().any(|s| matches!(s.kind, SuggestionKind::Sync)));
    }

    #[test]
    fn flags_software_update() {
        let processes = vec![process("macOS Software Update", Category::Update, 3_000_000.0, 0.0, 5, 1)];
        let suggestions = build(&processes);
        assert!(suggestions.iter().any(|s| matches!(s.kind, SuggestionKind::Update)));
    }

    #[test]
    fn ignores_background_chatter_below_sustain_threshold() {
        let processes = vec![process("mystery-daemon", Category::Unrecognized, 50_000.0, 0.0, 1, 1)];
        let suggestions = build(&processes);
        assert!(!suggestions.iter().any(|s| matches!(s.kind, SuggestionKind::Background)));
    }

    #[test]
    fn flags_sustained_background_chatter() {
        let processes = vec![process("mystery-daemon", Category::Unrecognized, 50_000.0, 0.0, 4, 1)];
        let suggestions = build(&processes);
        assert!(suggestions.iter().any(|s| matches!(s.kind, SuggestionKind::Background)));
    }

    #[test]
    fn notes_browser_helper_aggregation() {
        let processes = vec![process("Google Chrome", Category::App, 100_000.0, 0.0, 5, 4)];
        let suggestions = build(&processes);
        assert!(suggestions.iter().any(|s| matches!(s.kind, SuggestionKind::Info)));
    }
}
