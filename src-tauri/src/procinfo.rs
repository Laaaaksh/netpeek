//! Reads full command lines for local processes via `/bin/ps`, used only to
//! classify Chromium browser helper processes by their `--type=` flag (see
//! `breakdown.rs`). `nettop` reports a process's name but not its argv, and
//! Chrome's helper processes all share the same generic name, so this is
//! the only source for telling a renderer apart from the GPU process or an
//! extension.
//!
//! Like `nettop`, `ps` is a local, read-only system binary reading the
//! local process table - this does not add any outbound network access.

use std::collections::HashMap;
use std::process::Command;

pub const PS_PATH: &str = "/bin/ps";

/// Snapshots `pid -> full command line` for every process currently
/// visible to this user. Returns an empty map (rather than erroring) if
/// `ps` can't be run, so a transient failure just means that tick's
/// breakdown falls back to "Browser support process" instead of crashing
/// the sampler.
pub fn snapshot_argv() -> HashMap<u32, String> {
    let output = match Command::new(PS_PATH).args(["-axwwo", "pid=,command="]).output() {
        Ok(output) if output.status.success() => output,
        _ => return HashMap::new(),
    };
    parse_ps_output(&String::from_utf8_lossy(&output.stdout))
}

fn parse_ps_output(text: &str) -> HashMap<u32, String> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim_start();
        let Some((pid_str, command)) = line.split_once(' ') else {
            continue;
        };
        let Ok(pid) = pid_str.parse::<u32>() else {
            continue;
        };
        map.insert(pid, command.trim_start().to_string());
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pid_and_command() {
        let text = "  123 /Applications/Google Chrome.app/.../Google Chrome Helper --type=renderer\n  456 /usr/sbin/mDNSResponder\n";
        let map = parse_ps_output(text);
        assert_eq!(
            map.get(&123).unwrap(),
            "/Applications/Google Chrome.app/.../Google Chrome Helper --type=renderer"
        );
        assert_eq!(map.get(&456).unwrap(), "/usr/sbin/mDNSResponder");
    }

    #[test]
    fn ignores_malformed_lines() {
        let text = "not-a-pid some command\n\n";
        let map = parse_ps_output(text);
        assert!(map.is_empty());
    }
}
