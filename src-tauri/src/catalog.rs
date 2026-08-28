//! Maps raw `nettop` process names to a friendly display name, a group key
//! (so browser helper processes collapse into one row under their parent
//! browser), and a category used by the suggestion rule engine.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// A recognizable foreground application (browsers, chat apps, etc).
    App,
    /// A background file-sync client (Dropbox, iCloud Drive, Google Drive, OneDrive).
    Sync,
    /// A system or App Store update downloading in the background.
    Update,
    /// Time Machine backing up over the network.
    Backup,
    /// Anything else with no known application window.
    Background,
}

pub struct CatalogEntry {
    pub display_name: String,
    pub category: Category,
}

/// Looks up which "group" a raw process name belongs to, returning a stable
/// group key plus the metadata for that group. Matching is prefix-based
/// against the raw nettop process name, which is itself already truncated
/// (nettop caps names at 20-ish characters), so prefixes are what's reliable.
pub fn classify(raw_name: &str) -> (String, CatalogEntry) {
    for (prefixes, group_key, display_name, category) in RULES {
        if prefixes.iter().any(|p| raw_name.starts_with(p)) {
            return (
                group_key.to_string(),
                CatalogEntry {
                    display_name: display_name.to_string(),
                    category: *category,
                },
            );
        }
    }

    // Unrecognized process: group per-process-name as-is, no known window.
    (
        raw_name.to_string(),
        CatalogEntry {
            display_name: raw_name.to_string(),
            category: Category::Background,
        },
    )
}

type Rule = (&'static [&'static str], &'static str, &'static str, Category);

const RULES: &[Rule] = &[
    // Browsers - every helper/renderer/GPU process prefix collapses into one row.
    (&["Google Chrome"], "google-chrome", "Google Chrome", Category::App),
    (&["Microsoft Edge"], "microsoft-edge", "Microsoft Edge", Category::App),
    (&["Brave Browser"], "brave-browser", "Brave", Category::App),
    (&["firefox", "Firefox"], "firefox", "Firefox", Category::App),
    (&["Arc"], "arc", "Arc", Category::App),
    (&["Safari", "com.apple.WebKit"], "safari", "Safari", Category::App),
    // Common foreground apps.
    (&["Slack"], "slack", "Slack", Category::App),
    (&["Discord"], "discord", "Discord", Category::App),
    (&["zoom.us"], "zoom", "Zoom", Category::App),
    (&["Spotify"], "spotify", "Spotify", Category::App),
    (&["Claude"], "claude-app", "Claude", Category::App),
    (&["Cursor"], "cursor", "Cursor", Category::App),
    (&["Mail"], "mail", "Mail", Category::App),
    (&["Messages"], "messages", "Messages", Category::App),
    // Background file sync.
    (&["Dropbox"], "dropbox", "Dropbox (file sync)", Category::Sync),
    (&["bird"], "icloud-drive", "iCloud Drive (file sync)", Category::Sync),
    (
        &["Google Drive", "GoogleDrive", "FileProvider"],
        "google-drive",
        "Google Drive (file sync)",
        Category::Sync,
    ),
    (&["OneDrive"], "onedrive", "OneDrive (file sync)", Category::Sync),
    // Updates.
    (
        &["softwareupdated"],
        "softwareupdated",
        "macOS Software Update",
        Category::Update,
    ),
    (&["appstoreagent"], "appstoreagent", "App Store", Category::Update),
    // Backup.
    (&["backupd"], "backupd", "Time Machine", Category::Backup),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_chrome_helpers_together() {
        let (key_a, _) = classify("Google Chrome");
        let (key_b, _) = classify("Google Chrome H");
        let (key_c, _) = classify("Google Chrome Helper (Renderer)");
        assert_eq!(key_a, key_b);
        assert_eq!(key_b, key_c);
    }

    #[test]
    fn recognizes_sync_clients() {
        let (_, entry) = classify("bird");
        assert_eq!(entry.category, Category::Sync);
    }

    #[test]
    fn falls_back_to_background_for_unknown_daemons() {
        let (key, entry) = classify("mDNSResponder");
        assert_eq!(key, "mDNSResponder");
        assert_eq!(entry.category, Category::Background);
    }
}
