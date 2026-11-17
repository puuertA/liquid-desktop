use serde::Serialize;
use std::time::Instant;
use sysinfo::{Networks, System};

#[derive(Clone, Serialize)]
pub struct Snapshot {
    pub cpu: Option<f32>,
    pub memory_used: u64,
    pub memory_total: u64,
    pub download: Option<f64>,
    pub upload: Option<f64>,
    pub interfaces: usize,
    pub sequence: u64,
}
pub struct Monitor {
    system: System,
    networks: Networks,
    last: Instant,
    sample: Option<Snapshot>,
}
impl Monitor {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_usage();
        Self {
            system,
            networks: Networks::new_with_refreshed_list(),
            last: Instant::now(),
            sample: None,
        }
    }
    pub fn sample(&mut self) -> Snapshot {
        let elapsed = self.last.elapsed().as_secs_f64();
        if elapsed < 1.0 {
            if let Some(sample) = &self.sample {
                return sample.clone();
            }
        }
        self.system.refresh_memory();
        let ready = elapsed >= 1.0;
        let mut cpu = None;
        let mut download = None;
        let mut upload = None;
        if ready {
            self.system.refresh_cpu_usage();
            self.networks.refresh(true);
            cpu = Some(self.system.global_cpu_usage().clamp(0.0, 100.0));
            if !self.networks.is_empty() {
                download = Some(
                    self.networks
                        .values()
                        .map(|n| n.received() as f64)
                        .sum::<f64>()
                        / elapsed,
                );
                upload = Some(
                    self.networks
                        .values()
                        .map(|n| n.transmitted() as f64)
                        .sum::<f64>()
                        / elapsed,
                );
            }
            self.last = Instant::now();
        }
        let snapshot = Snapshot {
            cpu,
            memory_used: self.system.used_memory(),
            memory_total: self.system.total_memory(),
            download,
            upload,
            interfaces: self.networks.len(),
            sequence: self.sample.as_ref().map_or(1, |s| s.sequence + 1),
        };
        self.sample = Some(snapshot.clone());
        snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Reads real Windows counters and waits for a CPU sample"]
    fn reports_real_memory_and_warmed_cpu() {
        let mut monitor = Monitor::new();
        let first = monitor.sample();
        assert!(first.memory_total > 0, "Windows should expose physical RAM");
        assert!(
            first.cpu.is_none(),
            "First baseline is not a measured CPU sample"
        );
        std::thread::sleep(std::time::Duration::from_millis(1200));
        let sample = monitor.sample();
        assert!(sample.memory_used <= sample.memory_total);
        assert!(sample
            .cpu
            .is_some_and(|v| v.is_finite() && (0.0..=100.0).contains(&v)));
        assert!(sample.download.is_none_or(|v| v.is_finite() && v >= 0.0));
        assert!(sample.upload.is_none_or(|v| v.is_finite() && v >= 0.0));
        assert_eq!(
            monitor.sample().sequence,
            sample.sequence,
            "Fast callers share cached sample"
        );
    }
}
