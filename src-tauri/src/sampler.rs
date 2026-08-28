//! Turns raw cumulative `nettop` byte counters into per-second rates.
//!
//! Speed is always a delta between two consecutive samples divided by the
//! elapsed time, never a cumulative total. This module owns the "previous
//! sample" state and handles the edge cases that come with reading live
//! system counters: a process disappearing, a pid being reused by an
//! unrelated process, a counter resetting, and the very first sample having
//! no baseline to diff against.

use crate::catalog::{classify, Category};
use crate::nettop::RawSample;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Below this combined rate a group isn't considered "active" for the
/// purposes of tracking sustained background chatter.
const ACTIVE_THRESHOLD_BPS: f64 = 1024.0;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessRate {
    pub group_key: String,
    pub display_name: String,
    pub category: Category,
    pub download_bps: f64,
    pub upload_bps: f64,
    pub pid_count: usize,
    /// Consecutive samples this group has had non-trivial traffic. Used by
    /// the "background chatter" suggestion rule to judge sustained activity.
    pub sustained_seconds: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub processes: Vec<ProcessRate>,
    pub total_download_bps: f64,
    pub total_upload_bps: f64,
    pub timestamp_ms: u64,
}

struct PrevCounter {
    bytes_in: u64,
    bytes_out: u64,
    at: Instant,
}

pub struct Sampler {
    prev: HashMap<u32, PrevCounter>,
    streaks: HashMap<String, u32>,
    has_baseline: bool,
}

impl Sampler {
    pub fn new() -> Self {
        Self {
            prev: HashMap::new(),
            streaks: HashMap::new(),
            has_baseline: false,
        }
    }

    /// Folds one raw nettop batch into the running state and returns a
    /// snapshot of current rates, or `None` when there isn't yet a prior
    /// sample to diff against (the very first tick).
    pub fn ingest(&mut self, raw: Vec<RawSample>) -> Option<Snapshot> {
        let now = Instant::now();
        let mut seen_pids: HashSet<u32> = HashSet::with_capacity(raw.len());
        struct GroupAcc {
            display_name: String,
            category: Category,
            download_bps: f64,
            upload_bps: f64,
            pid_count: usize,
        }
        let mut groups: HashMap<String, GroupAcc> = HashMap::new();

        for sample in &raw {
            seen_pids.insert(sample.pid);
            let rates = self.prev.get(&sample.pid).and_then(|prev| {
                let elapsed = now.duration_since(prev.at).as_secs_f64();
                if elapsed <= 0.0 {
                    return None;
                }
                if sample.bytes_in < prev.bytes_in || sample.bytes_out < prev.bytes_out {
                    // Counter went backwards: either it reset, or this pid
                    // was reused by a different process. Treat this sample
                    // as a fresh baseline instead of reporting a rate.
                    return None;
                }
                let down = (sample.bytes_in - prev.bytes_in) as f64 / elapsed;
                let up = (sample.bytes_out - prev.bytes_out) as f64 / elapsed;
                Some((down, up))
            });

            self.prev.insert(
                sample.pid,
                PrevCounter {
                    bytes_in: sample.bytes_in,
                    bytes_out: sample.bytes_out,
                    at: now,
                },
            );

            let Some((down, up)) = rates else {
                continue;
            };

            let (group_key, entry) = classify(&sample.name);
            let acc = groups.entry(group_key).or_insert_with(|| GroupAcc {
                display_name: entry.display_name.clone(),
                category: entry.category,
                download_bps: 0.0,
                upload_bps: 0.0,
                pid_count: 0,
            });
            acc.download_bps += down;
            acc.upload_bps += up;
            acc.pid_count += 1;
        }

        // Forget baselines for pids that vanished between samples.
        self.prev.retain(|pid, _| seen_pids.contains(pid));

        if !self.has_baseline {
            self.has_baseline = true;
            return None;
        }

        let mut next_streaks = HashMap::with_capacity(groups.len());
        let mut processes: Vec<ProcessRate> = groups
            .into_iter()
            .map(|(group_key, acc)| {
                let active = acc.download_bps + acc.upload_bps >= ACTIVE_THRESHOLD_BPS;
                let streak = if active {
                    self.streaks.get(&group_key).copied().unwrap_or(0) + 1
                } else {
                    0
                };
                next_streaks.insert(group_key.clone(), streak);
                ProcessRate {
                    group_key,
                    display_name: acc.display_name,
                    category: acc.category,
                    download_bps: acc.download_bps,
                    upload_bps: acc.upload_bps,
                    pid_count: acc.pid_count,
                    sustained_seconds: streak,
                }
            })
            .collect();
        self.streaks = next_streaks;

        processes.sort_by(|a, b| {
            let a_total = a.download_bps + a.upload_bps;
            let b_total = b.download_bps + b.upload_bps;
            b_total.total_cmp(&a_total)
        });

        let total_download_bps = processes.iter().map(|p| p.download_bps).sum();
        let total_upload_bps = processes.iter().map(|p| p.upload_bps).sum();
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Some(Snapshot {
            processes,
            total_download_bps,
            total_upload_bps,
            timestamp_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(name: &str, pid: u32, bytes_in: u64, bytes_out: u64) -> RawSample {
        RawSample {
            pid,
            name: name.to_string(),
            bytes_in,
            bytes_out,
        }
    }

    #[test]
    fn first_sample_has_no_baseline() {
        let mut sampler = Sampler::new();
        let snap = sampler.ingest(vec![raw("mDNSResponder", 479, 1000, 500)]);
        assert!(snap.is_none());
    }

    #[test]
    fn second_sample_produces_a_rate() {
        let mut sampler = Sampler::new();
        sampler.ingest(vec![raw("mDNSResponder", 479, 1000, 500)]);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let snap = sampler
            .ingest(vec![raw("mDNSResponder", 479, 2000, 1500)])
            .unwrap();
        assert_eq!(snap.processes.len(), 1);
        assert!(snap.processes[0].download_bps > 0.0);
        assert!(snap.processes[0].upload_bps > 0.0);
    }

    #[test]
    fn counter_reset_does_not_produce_a_spike() {
        let mut sampler = Sampler::new();
        sampler.ingest(vec![raw("java", 936, 5_000_000, 0)]);
        std::thread::sleep(std::time::Duration::from_millis(10));
        // Counter dropped below the previous value (process restarted / pid reused).
        let snap = sampler.ingest(vec![raw("java", 936, 100, 0)]).unwrap();
        assert!(snap.processes.is_empty());
    }

    #[test]
    fn vanished_process_is_forgotten() {
        let mut sampler = Sampler::new();
        sampler.ingest(vec![raw("curl", 100, 0, 0)]);
        std::thread::sleep(std::time::Duration::from_millis(10));
        sampler.ingest(vec![raw("curl", 100, 1000, 0)]).unwrap();
        // curl exits; a new unrelated process reuses pid 100 immediately.
        std::thread::sleep(std::time::Duration::from_millis(10));
        let snap = sampler.ingest(vec![raw("node", 100, 5, 0)]).unwrap();
        assert!(snap.processes.is_empty());
    }

    #[test]
    fn chrome_helpers_aggregate_into_one_group() {
        let mut sampler = Sampler::new();
        sampler.ingest(vec![
            raw("Google Chrome", 1, 0, 0),
            raw("Google Chrome H", 2, 0, 0),
        ]);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let snap = sampler
            .ingest(vec![
                raw("Google Chrome", 1, 1000, 0),
                raw("Google Chrome H", 2, 1000, 0),
            ])
            .unwrap();
        assert_eq!(snap.processes.len(), 1);
        assert_eq!(snap.processes[0].pid_count, 2);
        assert_eq!(snap.processes[0].display_name, "Google Chrome");
    }
}
