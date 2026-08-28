//! Classifies a browser helper process's job - page content, graphics,
//! a supporting service, or a browser extension - so a browser's aggregate
//! row can be expanded into something a user can act on.
//!
//! Chrome's own process name (what `nettop` reports) is generic for every
//! helper ("Google Chrome Helper (Renderer)" at best, often just truncated
//! further), so it can't tell renderer/GPU/utility apart on its own. What
//! does is the `--type=`/`--utility-sub-type=`/`--extension-process` flags
//! on the helper's full command line, which is why classification here
//! needs argv (see `procinfo.rs`) rather than just the nettop name.
//!
//! Chrome does not put an extension's name or ID on its renderer's argv
//! (that's only visible in Chrome's own Task Manager), so an extension
//! process is identified as "a browser extension" without saying which one
//! - see the module doc on `procinfo.rs` and the catalog's task-manager
//! hint for the honest hand-off.
//!
//! Safari's helpers, by contrast, are already named distinctly by process
//! name alone (`com.apple.WebKit.WebContent`/`.Networking`/`.GPU`), so no
//! argv lookup is needed there - but Safari exposes nothing that lets us
//! tell an extension's traffic apart from ordinary page content.
//!
//! One honest caveat, confirmed by running this against a live multi-tab
//! Chrome with an extension installed: modern Chrome funnels almost all
//! actual socket I/O - for ordinary tabs and for extensions alike - through
//! its single shared Network Service utility process, not through the
//! renderer that requested it. `nettop` (and `ps`, and Activity Monitor)
//! only see byte counts on whichever process actually holds the socket, so
//! in practice most of a Chrome row's traffic lands in the "Network
//! connections" bucket even when a renderer or extension is what's really
//! driving it. The classification here is still correct for whatever
//! process nettop *does* attribute bytes to; this is a limit of what's
//! observable from outside the browser, not a bug - the same limit that
//! makes the catalog's task-manager hand-off necessary in the first place.

pub struct HelperJob {
    pub label: String,
    pub is_extension: bool,
}

impl HelperJob {
    fn new(label: &str) -> Self {
        Self { label: label.to_string(), is_extension: false }
    }

    fn extension(label: &str) -> Self {
        Self { label: label.to_string(), is_extension: true }
    }
}

/// Classifies a Chromium-family (Chrome/Edge/Brave/Arc/Opera) helper from
/// its full command line.
pub fn classify_chromium_job(argv: &str) -> HelperJob {
    match flag_value(argv, "--type=").as_deref() {
        Some("renderer") => {
            if argv.contains("--extension-process") {
                HelperJob::extension("Browser extension")
            } else {
                HelperJob::new("Page content (open tabs)")
            }
        }
        Some("gpu-process") => HelperJob::new("Graphics"),
        Some("utility") => {
            let label = match flag_value(argv, "--utility-sub-type=") {
                Some(sub) if sub.contains("network") => "Network connections",
                Some(sub) if sub.contains("audio") => "Audio",
                Some(sub) if sub.contains("storage") => "Storage",
                Some(sub) if sub.contains("video_capture") => "Camera access",
                Some(sub) if sub.contains("data_decoder") => "Data decoding",
                _ => "Supporting service",
            };
            HelperJob::new(label)
        }
        Some(_other) => HelperJob::new("Browser support process"),
        // No --type= flag at all: this is the main browser process itself.
        None => HelperJob::new("Browser (tabs & windows)"),
    }
}

/// Classifies a Safari/WebKit helper from its (untruncated) process name.
/// Safari doesn't expose per-extension attribution, so extensions that run
/// in their own process surface under their own name rather than here.
pub fn classify_safari_job(raw_name: &str) -> HelperJob {
    if raw_name.contains("Networking") {
        HelperJob::new("Network connections")
    } else if raw_name.contains("GPU") {
        HelperJob::new("Graphics")
    } else if raw_name.contains("WebContent") {
        HelperJob::new("Page content (open tabs)")
    } else {
        HelperJob::new("Browser support process")
    }
}

fn flag_value(argv: &str, flag: &str) -> Option<String> {
    argv.split_whitespace()
        .find(|token| token.starts_with(flag))
        .map(|token| token[flag.len()..].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_renderer_as_page_content() {
        let job = classify_chromium_job("/Applications/Google Chrome.app/.../Google Chrome Helper (Renderer) --type=renderer --lang=en-US");
        assert_eq!(job.label, "Page content (open tabs)");
        assert!(!job.is_extension);
    }

    #[test]
    fn classifies_extension_renderer_distinctly() {
        let job = classify_chromium_job("... --type=renderer --extension-process --lang=en-US");
        assert_eq!(job.label, "Browser extension");
        assert!(job.is_extension);
    }

    #[test]
    fn classifies_gpu_process() {
        let job = classify_chromium_job("... --type=gpu-process --gpu-preferences=abc");
        assert_eq!(job.label, "Graphics");
    }

    #[test]
    fn classifies_network_utility_service() {
        let job = classify_chromium_job("... --type=utility --utility-sub-type=network.mojom.NetworkService");
        assert_eq!(job.label, "Network connections");
    }

    #[test]
    fn classifies_unknown_utility_as_supporting_service() {
        let job = classify_chromium_job("... --type=utility --utility-sub-type=some.new.Service");
        assert_eq!(job.label, "Supporting service");
    }

    #[test]
    fn classifies_main_process_with_no_type_flag() {
        let job = classify_chromium_job("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome");
        assert_eq!(job.label, "Browser (tabs & windows)");
    }

    #[test]
    fn classifies_safari_helpers_by_name() {
        assert_eq!(classify_safari_job("com.apple.WebKit.WebContent").label, "Page content (open tabs)");
        assert_eq!(classify_safari_job("com.apple.WebKit.Networking").label, "Network connections");
        assert_eq!(classify_safari_job("com.apple.WebKit.GPU").label, "Graphics");
        assert_eq!(classify_safari_job("Safari").label, "Browser support process");
    }
}
