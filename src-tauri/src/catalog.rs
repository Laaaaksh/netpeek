//! Maps raw `nettop` process names to everything the UI needs to explain
//! that process to someone with no technical background: a friendly display
//! name, a group key (so browser helper processes collapse into one row
//! under their parent browser), a category used by the suggestion rule
//! engine, a plain-English sentence saying what the process is, and a
//! verdict saying whether the user needs to do anything about it.
//!
//! The verdict is deliberately as prominent as the name: for most system
//! processes the honest answer is "this is macOS itself, leave it alone",
//! and that reassurance is the point of this module as much as the naming
//! is. Nothing here ever tells the user to open Terminal, use `sudo`, or
//! edit a system file - every verdict is achievable from a GUI, or is "do
//! nothing".

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
    /// A recognized macOS system/network daemon with no visible window.
    /// Distinct from `Unrecognized`: we know what this one is, and the
    /// answer is always "normal, leave it alone".
    System,
    /// A process with no visible window that Netpeek does not recognize.
    /// We never invent an explanation for these.
    Unrecognized,
}

/// How a group's helper processes can be broken down further. Populated
/// only for browsers where the breakdown is driven by real data (argv for
/// Chromium, process name for Safari) rather than a guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BreakdownKind {
    /// Chrome, Edge, Brave, Arc, Opera - classified via each helper's
    /// `--type=`/`--utility-sub-type=`/`--extension-process` argv flags.
    Chromium,
    /// Safari - classified via the WebKit helper's process name alone
    /// (`com.apple.WebKit.WebContent`/`.Networking`/`.GPU`). Safari does not
    /// expose enough to attribute traffic to a specific extension.
    Safari,
}

/// Below this a browser's total isn't worth pointing someone at its Task
/// Manager for - shared with the suggestion engine's own notability bar.
pub const NOTABLE_BPS: f64 = 20.0 * 1024.0;

pub struct CatalogEntry {
    pub display_name: String,
    pub category: Category,
    pub what_it_is: String,
    pub verdict: String,
    pub breakdown_kind: Option<BreakdownKind>,
    pub task_manager_hint: Option<String>,
}

/// Looks up which "group" a raw process name belongs to, returning a stable
/// group key plus the metadata for that group. Matching is prefix-based
/// against the raw nettop process name, which is itself already truncated
/// (nettop caps names at 20-ish characters), so prefixes are what's reliable.
pub fn classify(raw_name: &str) -> (String, CatalogEntry) {
    for rule in RULES {
        if rule.prefixes.iter().any(|p| raw_name.starts_with(p)) {
            return (
                rule.group_key.to_string(),
                CatalogEntry {
                    display_name: rule.display_name.to_string(),
                    category: rule.category,
                    what_it_is: rule.what_it_is.to_string(),
                    verdict: rule.verdict.to_string(),
                    breakdown_kind: rule.breakdown_kind,
                    task_manager_hint: rule.task_manager_hint.map(str::to_string),
                },
            );
        }
    }

    // Unrecognized process: group per-process-name as-is, no known window,
    // and no invented explanation.
    (
        raw_name.to_string(),
        CatalogEntry {
            display_name: raw_name.to_string(),
            category: Category::Unrecognized,
            what_it_is: "Netpeek doesn't recognize this process.".to_string(),
            verdict: "No verdict - we don't guess at what an unrecognized process does. If it's using a lot of data and the name means nothing to you, searching for it by name is the safest way to find out.".to_string(),
            breakdown_kind: None,
            task_manager_hint: None,
        },
    )
}

struct Rule {
    prefixes: &'static [&'static str],
    group_key: &'static str,
    display_name: &'static str,
    category: Category,
    what_it_is: &'static str,
    verdict: &'static str,
    breakdown_kind: Option<BreakdownKind>,
    task_manager_hint: Option<&'static str>,
}

const LEAVE_ALONE: &str = "This is macOS itself - normal, and not something you need to do anything about.";

const RULES: &[Rule] = &[
    // --- Browsers: every helper/renderer/GPU process prefix collapses into
    // one row. Chromium-family browsers and Safari get a real breakdown;
    // Firefox stays aggregated (its multiprocess model isn't distinguishable
    // by name or a stable argv convention the way Chromium's is).
    Rule {
        prefixes: &["Google Chrome"],
        group_key: "google-chrome",
        display_name: "Google Chrome",
        category: Category::App,
        what_it_is: "Your Google Chrome browser - covers every tab, extension, and background helper it runs.",
        verdict: "Normal while you're browsing. Expand this row to see what inside Chrome is using the data.",
        breakdown_kind: Some(BreakdownKind::Chromium),
        task_manager_hint: Some("Chrome's own Task Manager (Window > Task Manager) can name the exact tab or extension responsible - Netpeek can only see Chrome as a whole, not individual tabs."),
    },
    Rule {
        prefixes: &["Microsoft Edge"],
        group_key: "microsoft-edge",
        display_name: "Microsoft Edge",
        category: Category::App,
        what_it_is: "Your Microsoft Edge browser - covers every tab, extension, and background helper it runs.",
        verdict: "Normal while you're browsing. Expand this row to see what inside Edge is using the data.",
        breakdown_kind: Some(BreakdownKind::Chromium),
        task_manager_hint: Some("Edge's own Task Manager (Settings and more > More tools > Browser task manager) can name the exact tab or extension responsible - Netpeek can only see Edge as a whole, not individual tabs."),
    },
    Rule {
        prefixes: &["Brave Browser"],
        group_key: "brave-browser",
        display_name: "Brave",
        category: Category::App,
        what_it_is: "Your Brave browser - covers every tab, extension, and background helper it runs.",
        verdict: "Normal while you're browsing. Expand this row to see what inside Brave is using the data.",
        breakdown_kind: Some(BreakdownKind::Chromium),
        task_manager_hint: Some("Brave's own Task Manager (Window > Task Manager) can name the exact tab or extension responsible - Netpeek can only see Brave as a whole, not individual tabs."),
    },
    Rule {
        prefixes: &["firefox", "Firefox"],
        group_key: "firefox",
        display_name: "Firefox",
        category: Category::App,
        what_it_is: "Your Firefox browser - covers every tab and background helper it runs.",
        verdict: "Normal while you're browsing. Firefox doesn't expose which tab is responsible the way Chromium browsers do, so this row can't be broken down further here - Firefox's own Task Manager (about:performance) can help.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["Arc"],
        group_key: "arc",
        display_name: "Arc",
        category: Category::App,
        what_it_is: "Your Arc browser - covers every tab, extension, and background helper it runs.",
        verdict: "Normal while you're browsing. Expand this row to see what inside Arc is using the data.",
        breakdown_kind: Some(BreakdownKind::Chromium),
        task_manager_hint: Some("Arc doesn't offer an easy built-in Task Manager - closing tabs one at a time is the most reliable way to find the culprit. Netpeek can only see Arc as a whole, not individual tabs."),
    },
    Rule {
        prefixes: &["Opera"],
        group_key: "opera",
        display_name: "Opera",
        category: Category::App,
        what_it_is: "Your Opera browser - covers every tab, extension, and background helper it runs.",
        verdict: "Normal while you're browsing. Expand this row to see what inside Opera is using the data.",
        breakdown_kind: Some(BreakdownKind::Chromium),
        task_manager_hint: Some("Opera has its own Task Manager (look for it in Opera's menu) that can name the exact tab or extension responsible - Netpeek can only see Opera as a whole, not individual tabs."),
    },
    Rule {
        prefixes: &["Safari", "com.apple.WebKit"],
        group_key: "safari",
        display_name: "Safari",
        category: Category::App,
        what_it_is: "Your Safari browser - covers every tab and background helper it runs.",
        verdict: "Normal while you're browsing. Expand this row to see the kind of work inside Safari that's using the data.",
        breakdown_kind: Some(BreakdownKind::Safari),
        task_manager_hint: None,
    },
    // --- Mainstream chat/video apps.
    Rule {
        prefixes: &["Slack"],
        group_key: "slack",
        display_name: "Slack",
        category: Category::App,
        what_it_is: "Slack - team chat, calls, and file uploads/downloads.",
        verdict: "Normal while Slack is open, especially during a call or a big file transfer.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["Discord"],
        group_key: "discord",
        display_name: "Discord",
        category: Category::App,
        what_it_is: "Discord - chat, voice/video calls, and screen sharing.",
        verdict: "Normal while Discord is open, especially during a call.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["zoom.us"],
        group_key: "zoom",
        display_name: "Zoom",
        category: Category::App,
        what_it_is: "Zoom - your video call audio and video.",
        verdict: "Normal during a call; usage drops to nothing once you hang up.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["Microsoft Teams"],
        group_key: "microsoft-teams",
        display_name: "Microsoft Teams",
        category: Category::App,
        what_it_is: "Microsoft Teams - chat, calls, and file uploads/downloads.",
        verdict: "Normal while Teams is open, especially during a call.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["WhatsApp"],
        group_key: "whatsapp",
        display_name: "WhatsApp",
        category: Category::App,
        what_it_is: "WhatsApp - messages, calls, and media.",
        verdict: "Normal while WhatsApp is open, especially during a call.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["Telegram"],
        group_key: "telegram",
        display_name: "Telegram",
        category: Category::App,
        what_it_is: "Telegram - messages, calls, and media.",
        verdict: "Normal while Telegram is open.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["Signal"],
        group_key: "signal",
        display_name: "Signal",
        category: Category::App,
        what_it_is: "Signal - messages, calls, and media.",
        verdict: "Normal while Signal is open.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["Skype"],
        group_key: "skype",
        display_name: "Skype",
        category: Category::App,
        what_it_is: "Skype - chat and video calls.",
        verdict: "Normal during a call; usage drops to nothing once you hang up.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["FaceTime"],
        group_key: "facetime",
        display_name: "FaceTime",
        category: Category::App,
        what_it_is: "Apple's FaceTime - your video or audio call.",
        verdict: "Normal during a call; usage drops to nothing once you hang up.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    // --- Other common foreground apps.
    Rule {
        prefixes: &["Spotify"],
        group_key: "spotify",
        display_name: "Spotify",
        category: Category::App,
        what_it_is: "Spotify streaming music or a podcast.",
        verdict: "Normal while something is playing. Downloading tracks for offline listening will cut this down if you want to.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["Claude"],
        group_key: "claude-app",
        display_name: "Claude",
        category: Category::App,
        what_it_is: "The Claude desktop app talking to Anthropic's servers to answer your prompts.",
        verdict: "Normal while you're using it.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["Cursor"],
        group_key: "cursor",
        display_name: "Cursor",
        category: Category::App,
        what_it_is: "The Cursor code editor talking to its AI service to answer your prompts.",
        verdict: "Normal while you're using it.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["Mail"],
        group_key: "mail",
        display_name: "Mail",
        category: Category::App,
        what_it_is: "Apple Mail checking for and downloading email.",
        verdict: "Normal, especially right after a new message with attachments arrives.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["Messages"],
        group_key: "messages",
        display_name: "Messages",
        category: Category::App,
        what_it_is: "Apple Messages sending or receiving iMessages and their attachments.",
        verdict: "Normal.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    // --- Background file sync.
    Rule {
        prefixes: &["Dropbox"],
        group_key: "dropbox",
        display_name: "Dropbox (file sync)",
        category: Category::Sync,
        what_it_is: "Dropbox uploading or downloading your synced files.",
        verdict: "Safe to pause from Dropbox's menu bar icon if you need the speed right now.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["bird"],
        group_key: "icloud-drive",
        display_name: "iCloud Drive (file sync)",
        category: Category::Sync,
        what_it_is: "iCloud Drive uploading or downloading your synced files.",
        verdict: "Safe to leave running; if you need the bandwidth now, iCloud Drive syncing can be paused in System Settings > Apple ID > iCloud.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["Google Drive", "GoogleDrive", "FileProvider"],
        group_key: "google-drive",
        display_name: "Google Drive (file sync)",
        category: Category::Sync,
        what_it_is: "Google Drive uploading or downloading your synced files.",
        verdict: "Safe to pause from Google Drive's menu bar icon if you need the speed right now.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["OneDrive"],
        group_key: "onedrive",
        display_name: "OneDrive (file sync)",
        category: Category::Sync,
        what_it_is: "OneDrive uploading or downloading your synced files.",
        verdict: "Safe to pause from OneDrive's menu bar icon if you need the speed right now.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["Box Sync", "Box.app", "boxsync"],
        group_key: "box",
        display_name: "Box (file sync)",
        category: Category::Sync,
        what_it_is: "Box uploading or downloading your synced files.",
        verdict: "Safe to pause from Box's menu bar icon if you need the speed right now.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    // --- Updates.
    Rule {
        prefixes: &["softwareupdated"],
        group_key: "softwareupdated",
        display_name: "macOS Software Update",
        category: Category::Update,
        what_it_is: "macOS downloading a system update.",
        verdict: "It'll finish on its own; pause it in System Settings > General > Software Update if you need the bandwidth right now.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["appstoreagent"],
        group_key: "appstoreagent",
        display_name: "App Store",
        category: Category::Update,
        what_it_is: "The App Store downloading or updating an app.",
        verdict: "It'll finish on its own; you can pause an individual app's download from the App Store window if you need the bandwidth right now.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    // --- Backup.
    Rule {
        prefixes: &["backupd"],
        group_key: "backupd",
        display_name: "Time Machine",
        category: Category::Backup,
        what_it_is: "Time Machine backing up your Mac over the network.",
        verdict: "Safe to pause until later from the Time Machine menu bar icon or System Settings > General > Time Machine.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    // --- iCloud (beyond iCloud Drive, which is filed under Sync above).
    Rule {
        prefixes: &["cloudd"],
        group_key: "cloudd",
        display_name: "iCloud Sync",
        category: Category::System,
        what_it_is: "macOS syncing your iCloud data - things like Notes, Reminders, and Contacts - with Apple's servers.",
        verdict: LEAVE_ALONE,
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["cloudphotod", "photolibraryd", "PhotoAnalysisD"],
        group_key: "icloud-photos",
        display_name: "iCloud Photos",
        category: Category::System,
        what_it_is: "macOS uploading or downloading your photo library through iCloud Photos.",
        verdict: "Normal - safe to leave running, or pause iCloud Photos in System Settings > Apple ID > iCloud if you need the bandwidth right now.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    // --- macOS networking & sync daemons with no visible window.
    Rule {
        prefixes: &["mDNSResponder"],
        group_key: "mdnsresponder",
        display_name: "mDNSResponder",
        category: Category::System,
        what_it_is: "macOS looking for printers, AirPlay devices, and other things on your local network.",
        verdict: LEAVE_ALONE,
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["apsd"],
        group_key: "apsd",
        display_name: "Apple Push Notifications",
        category: Category::System,
        what_it_is: "macOS staying connected to Apple's servers so apps can receive push notifications.",
        verdict: LEAVE_ALONE,
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["trustd"],
        group_key: "trustd",
        display_name: "Certificate Checks",
        category: Category::System,
        what_it_is: "macOS checking that an app or website's security certificate is still valid and hasn't been revoked.",
        verdict: LEAVE_ALONE,
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["identityservicesd"],
        group_key: "identityservicesd",
        display_name: "iMessage & FaceTime Connection",
        category: Category::System,
        what_it_is: "macOS keeping iMessage and FaceTime signed in and ready to receive messages or calls.",
        verdict: LEAVE_ALONE,
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["nsurlsessiond"],
        group_key: "nsurlsessiond",
        display_name: "Background App Downloads",
        category: Category::System,
        what_it_is: "macOS handling a background download or upload on behalf of one of your apps.",
        verdict: LEAVE_ALONE,
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["rapportd"],
        group_key: "rapportd",
        display_name: "Handoff & Continuity",
        category: Category::System,
        what_it_is: "macOS talking to your other Apple devices for Handoff and Continuity features (like starting an email on your iPhone and finishing it here).",
        verdict: LEAVE_ALONE,
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["sharingd"],
        group_key: "sharingd",
        display_name: "AirDrop & Handoff Discovery",
        category: Category::System,
        what_it_is: "macOS looking for nearby Apple devices for AirDrop and Handoff.",
        verdict: LEAVE_ALONE,
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["AirPlayXPCHelper"],
        group_key: "airplay",
        display_name: "AirPlay Discovery",
        category: Category::System,
        what_it_is: "macOS looking for AirPlay speakers and TVs on your network.",
        verdict: LEAVE_ALONE,
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["configd"],
        group_key: "configd",
        display_name: "Network Configuration",
        category: Category::System,
        what_it_is: "macOS managing your Wi-Fi, VPN, and other network connections.",
        verdict: LEAVE_ALONE,
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["netbiosd"],
        group_key: "netbiosd",
        display_name: "Windows File Sharing Discovery",
        category: Category::System,
        what_it_is: "macOS looking for Windows PCs and shared folders on your network.",
        verdict: LEAVE_ALONE,
        breakdown_kind: None,
        task_manager_hint: None,
    },
    // --- Package managers & dev tools.
    Rule {
        prefixes: &["brew"],
        group_key: "homebrew",
        display_name: "Homebrew",
        category: Category::App,
        what_it_is: "Homebrew downloading or updating a package you asked it to install.",
        verdict: "Normal - it'll finish on its own once the download completes.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["npm"],
        group_key: "npm",
        display_name: "npm",
        category: Category::App,
        what_it_is: "npm downloading JavaScript packages for a project.",
        verdict: "Normal - it'll finish on its own once the download completes.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["pip3", "pip"],
        group_key: "pip",
        display_name: "pip",
        category: Category::App,
        what_it_is: "pip downloading Python packages for a project.",
        verdict: "Normal - it'll finish on its own once the download completes.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["git"],
        group_key: "git",
        display_name: "Git",
        category: Category::App,
        what_it_is: "Git talking to a remote code repository (a clone, pull, fetch, or push).",
        verdict: "Normal - it'll finish on its own once the transfer completes.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
    Rule {
        prefixes: &["com.docker"],
        group_key: "docker",
        display_name: "Docker",
        category: Category::App,
        what_it_is: "Docker Desktop downloading a container image or syncing containers.",
        verdict: "Normal while a pull or build is running; it'll finish on its own.",
        breakdown_kind: None,
        task_manager_hint: None,
    },
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
    fn recognizes_known_system_daemons_with_an_explanation() {
        let (_, entry) = classify("mDNSResponder");
        assert_eq!(entry.category, Category::System);
        assert!(!entry.what_it_is.is_empty());
        assert!(!entry.verdict.is_empty());
    }

    #[test]
    fn falls_back_to_unrecognized_for_unknown_daemons() {
        let (key, entry) = classify("some_never_before_seen_daemon");
        assert_eq!(key, "some_never_before_seen_daemon");
        assert_eq!(entry.category, Category::Unrecognized);
        assert!(entry.what_it_is.contains("doesn't recognize"));
    }

    #[test]
    fn chromium_family_browsers_carry_a_breakdown_kind() {
        for name in ["Google Chrome", "Microsoft Edge", "Brave Browser", "Arc", "Opera"] {
            let (_, entry) = classify(name);
            assert_eq!(entry.breakdown_kind, Some(BreakdownKind::Chromium), "{name}");
        }
    }

    #[test]
    fn safari_carries_a_safari_breakdown_kind_and_no_task_manager_hint() {
        let (_, entry) = classify("com.apple.WebKit.WebContent");
        assert_eq!(entry.breakdown_kind, Some(BreakdownKind::Safari));
        assert!(entry.task_manager_hint.is_none());
    }

    #[test]
    fn firefox_has_no_breakdown() {
        let (_, entry) = classify("firefox");
        assert!(entry.breakdown_kind.is_none());
        assert!(entry.task_manager_hint.is_none());
    }

    #[test]
    fn every_rule_has_non_empty_copy() {
        for rule in RULES {
            assert!(!rule.what_it_is.is_empty(), "{}", rule.group_key);
            assert!(!rule.verdict.is_empty(), "{}", rule.group_key);
        }
    }
}
