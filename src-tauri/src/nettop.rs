//! Spawns `/usr/bin/nettop` under a pseudo-tty and streams parsed per-process
//! byte counters back to the sampler.
//!
//! `nettop` fully buffers its stdout when it isn't attached to a terminal, so
//! piping it directly yields output in multi-second bursts instead of once a
//! second. Running it under a PTY makes it think it has a terminal, which
//! keeps it flushing after every sample.

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{BufRead, BufReader};
use std::sync::mpsc::Sender;
use std::time::Duration;

pub const NETTOP_PATH: &str = "/usr/bin/nettop";

#[derive(Debug, Clone)]
pub struct RawSample {
    pub pid: u32,
    pub name: String,
    pub bytes_in: u64,
    pub bytes_out: u64,
}

/// Runs the nettop reader loop on a dedicated thread for the lifetime of the
/// app, restarting it if the child process ever exits.
pub fn spawn_reader(tx: Sender<Vec<RawSample>>) {
    std::thread::spawn(move || loop {
        if let Err(err) = run_once(&tx) {
            log::error!("nettop reader stopped ({err}); restarting shortly");
        }
        std::thread::sleep(Duration::from_secs(2));
    });
}

fn run_once(tx: &Sender<Vec<RawSample>>) -> anyhow::Result<()> {
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 200,
        cols: 250,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let mut cmd = CommandBuilder::new(NETTOP_PATH);
    cmd.args(["-P", "-L", "0", "-x", "-J", "bytes_in,bytes_out"]);

    let mut child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);

    let reader = pair.master.try_clone_reader()?;
    let mut lines = BufReader::new(reader);
    let mut batch: Vec<RawSample> = Vec::new();
    let mut line = String::new();
    let mut seen_first_header = false;

    loop {
        line.clear();
        let bytes_read = lines.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            continue;
        }

        if is_header(trimmed) {
            if seen_first_header && !batch.is_empty() {
                let _ = tx.send(std::mem::take(&mut batch));
            }
            seen_first_header = true;
            continue;
        }

        if let Some(sample) = parse_data_line(trimmed) {
            batch.push(sample);
        }
    }

    let _ = child.wait();
    Ok(())
}

fn is_header(line: &str) -> bool {
    line.starts_with(',')
}

/// Parses a data row shaped like `Google Chrome H.833,16848890,1757381,`
/// (name, pid separated by the final `.`, followed by cumulative in/out bytes).
fn parse_data_line(line: &str) -> Option<RawSample> {
    let mut fields = line.split(',');
    let name_pid = fields.next()?;
    let bytes_in: u64 = fields.next()?.parse().ok()?;
    let bytes_out: u64 = fields.next()?.parse().ok()?;

    let dot_idx = name_pid.rfind('.')?;
    let pid: u32 = name_pid[dot_idx + 1..].parse().ok()?;
    let name = name_pid[..dot_idx].to_string();

    Some(RawSample {
        pid,
        name,
        bytes_in,
        bytes_out,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_normal_row() {
        let sample = parse_data_line("Google Chrome H.833,16848890,1757381,").unwrap();
        assert_eq!(sample.name, "Google Chrome H");
        assert_eq!(sample.pid, 833);
        assert_eq!(sample.bytes_in, 16848890);
        assert_eq!(sample.bytes_out, 1757381);
    }

    #[test]
    fn parses_a_dotted_process_name() {
        let sample = parse_data_line("com.docker.back.40386,0,0,").unwrap();
        assert_eq!(sample.name, "com.docker.back");
        assert_eq!(sample.pid, 40386);
    }

    #[test]
    fn header_row_is_not_data() {
        assert!(parse_data_line(",bytes_in,bytes_out,").is_none());
        assert!(is_header(",bytes_in,bytes_out,"));
    }
}
