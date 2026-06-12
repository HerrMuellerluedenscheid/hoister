//! Opt-in container resource-usage collection. When the operator sets
//! `HOISTER_REPORT_METRICS=true`, this samples Docker's `stats` endpoint for
//! every tracked container on a fixed interval and ships CPU/memory figures
//! to the controller, which persists them as a time series for graphing.
//!
//! Kept separate from `monitor` because it runs on a coarser cadence (once a
//! minute vs the 5s state heartbeat): a stats sample is comparatively
//! expensive and per-minute resolution is plenty for trend graphs.
//!
//! CPU is reported as the *average* utilization over each sample interval, not
//! an instantaneous snapshot. cgroup CPU usage is a monotonic counter, so the
//! delta of `total_usage` (and of `system_cpu_usage`) between two consecutive
//! ticks yields the exact mean CPU% over the whole minute — see
//! [`windowed_cpu_pct`]. Memory and the network/storage counters are taken as
//! the latest reading on each tick.

use crate::HoisterError;
use crate::docker::get_service_identifier;
use crate::monitor::list_tracked_containers;
use bollard::Docker;
use bollard::models::ContainerStatsResponse;
use bollard::query_parameters::StatsOptionsBuilder;
use futures_util::StreamExt;
use hoister_shared::wire::{ContainerMetricSample, PostContainerMetricsRequest};
use hoister_shared::{HostName, ProjectName, ServiceName};
use log::{debug, error, info};
use reqwest::Url;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time;

/// How often we sample stats. Decoupled from the 5s state monitor — per-minute
/// resolution keeps the stored time series small while still showing trends.
const SAMPLE_INTERVAL: Duration = Duration::from_secs(60);

/// Cumulative CPU counters carried between ticks so we can compute the average
/// utilization over the whole interval rather than the ~1s window inside a
/// single stats frame. All three come straight out of one Docker stats reading.
#[derive(Clone, Copy)]
struct CpuCounters {
    /// Total CPU time the container has consumed since start (nanoseconds).
    total: u64,
    /// Total system-wide CPU time over the same epoch (nanoseconds).
    system: u64,
    /// Number of CPUs visible to the container at this reading.
    online_cpus: u32,
}

async fn collect_samples(
    project_name: &ProjectName,
    docker: &Docker,
    prev_cpu: &mut HashMap<ServiceName, CpuCounters>,
) -> Result<HashMap<ServiceName, ContainerMetricSample>, HoisterError> {
    let containers = list_tracked_containers(project_name, docker).await?;

    let mut samples = HashMap::new();
    let mut seen = std::collections::HashSet::new();
    for container in containers {
        let Some(container_id) = &container.id else {
            continue;
        };
        // Only running containers produce meaningful stats; a stopped
        // container yields an immediately-closing stream.
        let service_identifier = match get_service_identifier(docker, container_id).await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to resolve service identifier for {container_id}: {e}");
                continue;
            }
        };

        let options = StatsOptionsBuilder::default().stream(false).build();
        let mut stream = docker.stats(container_id, Some(options)).take(1);
        match stream.next().await {
            Some(Ok(stats)) => {
                let prev = prev_cpu.get(&service_identifier).copied();
                if let Some((sample, cur)) = sample_from_stats(&stats, prev) {
                    seen.insert(service_identifier.clone());
                    prev_cpu.insert(service_identifier.clone(), cur);
                    samples.insert(service_identifier, sample);
                }
            }
            Some(Err(e)) => debug!("stats unavailable for {container_id}: {e}"),
            None => debug!("no stats sample for {container_id} (likely not running)"),
        }
    }

    // Drop carried CPU state for services that are no longer tracked so the
    // map can't grow without bound across redeploys.
    prev_cpu.retain(|service, _| seen.contains(service));

    Ok(samples)
}

/// Turn a raw Docker stats sample into the wire sample, computing CPU% as the
/// average over the interval since `prev` (the counters captured on the last
/// tick for this service). On the first observation, or after a counter reset,
/// it falls back to the in-frame precpu delta so the tick still reports a
/// figure. Returns `None` when the payload lacks the fields we need (e.g. a
/// container that just stopped, or a Windows container), otherwise the wire
/// sample paired with the cumulative counters to carry into the next tick.
fn sample_from_stats(
    stats: &ContainerStatsResponse,
    prev: Option<CpuCounters>,
) -> Option<(ContainerMetricSample, CpuCounters)> {
    let cpu = stats.cpu_stats.as_ref()?;
    let precpu = stats.precpu_stats.as_ref();
    let mem = stats.memory_stats.as_ref()?;

    let cpu_total = cpu.cpu_usage.as_ref().and_then(|u| u.total_usage)?;
    let pre_cpu_total = precpu
        .and_then(|p| p.cpu_usage.as_ref())
        .and_then(|u| u.total_usage)
        .unwrap_or(0);
    let system = cpu.system_cpu_usage?;
    let pre_system = precpu.and_then(|p| p.system_cpu_usage).unwrap_or(0);
    let online_cpus = cpu
        .online_cpus
        .or_else(|| {
            cpu.cpu_usage
                .as_ref()
                .and_then(|u| u.percpu_usage.as_ref())
                .map(|v| v.len() as u32)
        })
        .unwrap_or(1)
        .max(1);

    let cur = CpuCounters {
        total: cpu_total,
        system,
        online_cpus,
    };
    // Average over the inter-tick window when we have a prior; otherwise the
    // single-frame precpu delta as a one-tick bootstrap.
    let in_frame = cpu_percent(cpu_total, pre_cpu_total, system, pre_system, online_cpus);
    let cpu_pct = windowed_cpu_pct(prev.as_ref(), &cur, in_frame);

    let mem_limit_bytes = mem.limit.unwrap_or(0);
    let mem_bytes = mem.usage.map(|usage| {
        // Subtract page cache so the figure matches `docker stats` (RSS-like).
        // cgroup v1 exposes `cache`; cgroup v2 exposes `inactive_file`.
        let cache = mem
            .stats
            .as_ref()
            .and_then(|s| s.get("cache").or_else(|| s.get("inactive_file")).copied())
            .unwrap_or(0);
        usage.saturating_sub(cache)
    })?;

    let (net_rx_bytes, net_tx_bytes) = stats
        .networks
        .as_ref()
        .map(|nets| {
            nets.values().fold((0u64, 0u64), |(rx, tx), iface| {
                (
                    rx + iface.rx_bytes.unwrap_or(0),
                    tx + iface.tx_bytes.unwrap_or(0),
                )
            })
        })
        .unwrap_or((0, 0));

    let storage_read_bytes = stats
        .storage_stats
        .as_ref()
        .and_then(|s| s.read_size_bytes)
        .unwrap_or(0);
    let storage_write_bytes = stats
        .storage_stats
        .as_ref()
        .and_then(|s| s.write_size_bytes)
        .unwrap_or(0);

    Some((
        ContainerMetricSample {
            cpu_pct,
            mem_bytes,
            mem_limit_bytes,
            net_rx_bytes,
            net_tx_bytes,
            storage_read_bytes,
            storage_write_bytes,
        },
        cur,
    ))
}

/// Average CPU% over the interval between `prev` and `cur` cumulative counters.
/// Falls back to `in_frame` (the single-frame precpu delta) when there is no
/// usable prior — the first observation of a service, or a counter reset where
/// the totals went backwards (container restart).
fn windowed_cpu_pct(prev: Option<&CpuCounters>, cur: &CpuCounters, in_frame: f64) -> f64 {
    match prev {
        Some(prev) if cur.total >= prev.total && cur.system >= prev.system => cpu_percent(
            cur.total,
            prev.total,
            cur.system,
            prev.system,
            cur.online_cpus,
        ),
        _ => in_frame,
    }
}

/// CPU percentage using the standard Docker formula:
/// `(cpu_delta / system_delta) * online_cpus * 100`. Returns 0 when there is
/// no system delta (first sample after start, or a stopped container).
fn cpu_percent(
    cpu_total: u64,
    pre_cpu_total: u64,
    system: u64,
    pre_system: u64,
    online_cpus: u32,
) -> f64 {
    let cpu_delta = cpu_total.saturating_sub(pre_cpu_total) as f64;
    let system_delta = system.saturating_sub(pre_system) as f64;
    if system_delta <= 0.0 || cpu_delta < 0.0 {
        return 0.0;
    }
    (cpu_delta / system_delta) * online_cpus as f64 * 100.0
}

async fn send_to_backend(
    client: &reqwest::Client,
    controller_url: &Url,
    token: Option<&str>,
    project_name: ProjectName,
    hostname: HostName,
    samples: &HashMap<ServiceName, ContainerMetricSample>,
) -> Result<(), reqwest::Error> {
    let url = controller_url
        .join(format!("container/metrics/{}/{}", hostname.0, project_name.0).as_str())
        .expect("failed to join url");

    debug!(
        "sending {} metric samples to {} (services: {})",
        samples.len(),
        url,
        samples
            .keys()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let request = PostContainerMetricsRequest {
        project_name,
        payload: samples.clone(),
    };

    let mut req = client.post(url).json(&request);
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let response = req.send().await?;
    debug!("metrics POST status: {}", response.status());
    response.error_for_status()?;
    Ok(())
}

pub(crate) async fn start(
    controller_url: &Url,
    token: Option<String>,
    project_name: ProjectName,
    hostname: HostName,
    client: reqwest::Client,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    info!("Starting metrics collector (interval: {SAMPLE_INTERVAL:?})");
    let docker = Docker::connect_with_socket_defaults()?;
    let mut interval = time::interval(SAMPLE_INTERVAL);
    // Cumulative CPU counters from the previous tick, per service, so CPU% is
    // an average over the whole interval rather than a sub-second snapshot.
    let mut prev_cpu: HashMap<ServiceName, CpuCounters> = HashMap::new();

    loop {
        interval.tick().await;

        match collect_samples(&project_name, &docker, &mut prev_cpu).await {
            Ok(samples) if samples.is_empty() => {
                debug!("No metrics samples collected this tick");
            }
            Ok(samples) => {
                if let Err(e) = send_to_backend(
                    &client,
                    controller_url,
                    token.as_deref(),
                    project_name.clone(),
                    hostname.clone(),
                    &samples,
                )
                .await
                {
                    error!("Failed to send metrics to backend: {e}");
                } else {
                    debug!("Sent metrics for {} services", samples.len());
                }
            }
            Err(e) => error!("Error collecting metrics: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_percent_typical() {
        // 1 CPU, cpu used 10ms of a 100ms system window -> 10%.
        let pct = cpu_percent(10_000_000, 0, 100_000_000, 0, 1);
        assert!((pct - 10.0).abs() < 1e-6, "got {pct}");
    }

    #[test]
    fn cpu_percent_scales_with_cores() {
        // Fully busy on all 4 cores: container's cpu_delta equals the
        // host-wide system_delta (summed across cores) -> 400%.
        let pct = cpu_percent(400, 0, 400, 0, 4);
        assert!((pct - 400.0).abs() < 1e-6, "got {pct}");
    }

    #[test]
    fn cpu_percent_zero_system_delta() {
        // No system delta (e.g. first sample) -> 0, never NaN/inf.
        let pct = cpu_percent(50, 0, 100, 100, 2);
        assert_eq!(pct, 0.0);
    }

    #[test]
    fn cpu_percent_counter_reset() {
        // Counters going backwards (container restart) must not go negative.
        let pct = cpu_percent(5, 10, 200, 100, 1);
        assert_eq!(pct, 0.0);
    }

    #[test]
    fn windowed_uses_inter_tick_delta_when_prior_present() {
        // 1 CPU; over the interval the container used 30ms of a 100ms system
        // window -> 30% average, independent of the in-frame fallback value.
        let prev = CpuCounters {
            total: 10_000_000,
            system: 1_000_000_000,
            online_cpus: 1,
        };
        let cur = CpuCounters {
            total: 40_000_000,
            system: 1_100_000_000,
            online_cpus: 1,
        };
        let pct = windowed_cpu_pct(Some(&prev), &cur, 99.0);
        assert!((pct - 30.0).abs() < 1e-6, "got {pct}");
    }

    #[test]
    fn windowed_falls_back_on_first_observation() {
        let cur = CpuCounters {
            total: 40_000_000,
            system: 1_100_000_000,
            online_cpus: 1,
        };
        let pct = windowed_cpu_pct(None, &cur, 12.5);
        assert_eq!(pct, 12.5);
    }

    #[test]
    fn windowed_falls_back_on_counter_reset() {
        // Container restarted: current totals are below the carried prior, so
        // a windowed delta would be meaningless -> use the in-frame fallback.
        let prev = CpuCounters {
            total: 40_000_000,
            system: 1_100_000_000,
            online_cpus: 1,
        };
        let cur = CpuCounters {
            total: 5_000_000,
            system: 1_200_000_000,
            online_cpus: 1,
        };
        let pct = windowed_cpu_pct(Some(&prev), &cur, 7.5);
        assert_eq!(pct, 7.5);
    }
}
