//! Opt-in container resource-usage collection. When the operator sets
//! `HOISTER_REPORT_METRICS=true`, this samples Docker's `stats` endpoint for
//! every tracked container on a fixed interval and ships CPU/memory figures
//! to the controller, which persists them as a time series for graphing.
//!
//! Kept separate from `monitor` because it runs on a coarser cadence (once a
//! minute vs the 5s state heartbeat): a stats sample is comparatively
//! expensive and per-minute resolution is plenty for trend graphs.

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

async fn collect_samples(
    project_name: &ProjectName,
    docker: &Docker,
) -> Result<HashMap<ServiceName, ContainerMetricSample>, HoisterError> {
    let containers = list_tracked_containers(project_name, docker).await?;

    let mut samples = HashMap::new();
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
                if let Some(sample) = sample_from_stats(&stats) {
                    samples.insert(service_identifier, sample);
                }
            }
            Some(Err(e)) => debug!("stats unavailable for {container_id}: {e}"),
            None => debug!("no stats sample for {container_id} (likely not running)"),
        }
    }

    Ok(samples)
}

/// Turn a raw Docker stats sample into the wire sample, computing CPU% the way
/// `docker stats` does. Returns `None` when the payload lacks the deltas we
/// need (e.g. a container that just stopped, or a Windows container).
fn sample_from_stats(stats: &ContainerStatsResponse) -> Option<ContainerMetricSample> {
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

    let cpu_pct = cpu_percent(cpu_total, pre_cpu_total, system, pre_system, online_cpus);

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

    Some(ContainerMetricSample {
        cpu_pct,
        mem_bytes,
        mem_limit_bytes,
    })
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

    loop {
        interval.tick().await;

        match collect_samples(&project_name, &docker).await {
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
}
