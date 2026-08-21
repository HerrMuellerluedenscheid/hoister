//! Metric alert rules and their temporal evaluation, shared between the agent
//! (rules from the TOML config, evaluated standalone) and the cloud controller
//! (rules from the dashboard, evaluated on ingested samples). Both feed
//! per-minute [`ContainerMetricSample`] readings into the same state machine so
//! the firing semantics are identical everywhere.
//!
//! Temporal semantics (Prometheus-style `for`, with symmetric hysteresis):
//! a rule *fires* only once the watched value has been at/above the threshold
//! continuously for `for_seconds` — a single spiky sample only moves the rule
//! to *pending*. While firing, the rule *resolves* only once the value has
//! stayed below the threshold for `for_seconds` again, so a value oscillating
//! around the threshold cannot ping-pong notifications every sample. While a
//! rule keeps firing, `cooldown_seconds > 0` re-notifies at most that often;
//! `0` notifies once per episode.
//!
//! Timestamps are plain unix seconds and durations plain seconds: both sides
//! need to persist or serialize the state (DB columns on the controller), and
//! `i64`/`u64` keep this crate free of a clock/time dependency.

use crate::wire::ContainerMetricSample;
use chatterbox::message::Message;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Which figure of a [`ContainerMetricSample`] an alert rule watches.
#[derive(TS, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AlertMetric {
    /// CPU percentage as reported by `docker stats`: 0..(100 × cores), already
    /// averaged over the sample interval by the agent.
    CpuPct,
    /// Memory usage as a percentage of the container's memory limit. Yields no
    /// value (and thus never fires) when the container has no limit.
    MemPct,
    /// Absolute memory usage in bytes.
    MemBytes,
}

impl AlertMetric {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertMetric::CpuPct => "cpu_pct",
            AlertMetric::MemPct => "mem_pct",
            AlertMetric::MemBytes => "mem_bytes",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "cpu_pct" => Some(Self::CpuPct),
            "mem_pct" => Some(Self::MemPct),
            "mem_bytes" => Some(Self::MemBytes),
            _ => None,
        }
    }

    /// Extract the watched value from a sample. `None` when the sample cannot
    /// produce it (memory percentage without a memory limit).
    pub fn value(&self, sample: &ContainerMetricSample) -> Option<f64> {
        match self {
            AlertMetric::CpuPct => Some(sample.cpu_pct),
            AlertMetric::MemPct => {
                if sample.mem_limit_bytes == 0 {
                    None
                } else {
                    Some(sample.mem_bytes as f64 / sample.mem_limit_bytes as f64 * 100.0)
                }
            }
            AlertMetric::MemBytes => Some(sample.mem_bytes as f64),
        }
    }

    /// Human-readable rendering of a value of this metric for notification
    /// bodies: percentages with one decimal, byte counts in binary units.
    pub fn format_value(&self, value: f64) -> String {
        match self {
            AlertMetric::CpuPct | AlertMetric::MemPct => format!("{value:.1}%"),
            AlertMetric::MemBytes => format_bytes(value),
        }
    }

    /// Short human label used in notification titles ("cpu", "memory").
    pub fn label(&self) -> &'static str {
        match self {
            AlertMetric::CpuPct => "cpu",
            AlertMetric::MemPct | AlertMetric::MemBytes => "memory",
        }
    }
}

fn format_bytes(value: f64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = value.max(0.0);
    let mut unit = 0;
    while v >= 1024.0 && unit < UNITS.len() - 1 {
        v /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{v:.0} {}", UNITS[unit])
    } else {
        format!("{v:.1} {}", UNITS[unit])
    }
}

/// Render a duration in seconds compactly for notification bodies and config
/// echoes: "90s" -> "1m 30s", "3600s" -> "1h".
pub fn format_duration_secs(secs: u64) -> String {
    if secs == 0 {
        return "0s".to_string();
    }
    let (h, m, s) = (secs / 3600, (secs % 3600) / 60, secs % 60);
    let mut parts = Vec::new();
    if h > 0 {
        parts.push(format!("{h}h"));
    }
    if m > 0 {
        parts.push(format!("{m}m"));
    }
    if s > 0 {
        parts.push(format!("{s}s"));
    }
    parts.join(" ")
}

/// Parse a human duration into seconds: plain integers are seconds, and the
/// suffixes `s`, `m`, `h`, `d` are understood ("30s", "5m", "2h", "1d").
pub fn parse_duration_secs(raw: &str) -> Option<u64> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let (number, factor) = match raw.chars().last() {
        Some('s') => (&raw[..raw.len() - 1], 1),
        Some('m') => (&raw[..raw.len() - 1], 60),
        Some('h') => (&raw[..raw.len() - 1], 3600),
        Some('d') => (&raw[..raw.len() - 1], 86_400),
        Some(c) if c.is_ascii_digit() => (raw, 1),
        _ => return None,
    };
    number.trim().parse::<u64>().ok().map(|n| n * factor)
}

/// Threshold and temporal requirements of one alert rule, in the units the
/// state machine consumes. How rules are configured (TOML on the agent, DB
/// rows on the controller) is up to the caller.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AlertCondition {
    pub metric: AlertMetric,
    /// Firing threshold, in the metric's unit (percent or bytes). A sample at
    /// or above the threshold counts as breaching.
    pub threshold: f64,
    /// The value must breach continuously for at least this long before the
    /// rule fires; it must then stay below for the same duration to resolve.
    /// `0` fires and resolves on a single sample.
    pub for_seconds: u64,
    /// Minimum seconds between repeat notifications while the rule keeps
    /// firing. `0` notifies once per firing episode.
    pub cooldown_seconds: u64,
}

/// Where one (rule, service) pair currently stands. Fields are public so the
/// controller can persist and rehydrate the state across restarts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum AlertState {
    #[default]
    Ok,
    /// Breaching, but not yet long enough to fire.
    Pending { since: i64 },
    Firing {
        since: i64,
        /// When the last notification for this episode went out — drives the
        /// cooldown for repeats.
        last_notified: i64,
        /// Set while the value has dropped below the threshold: start of the
        /// candidate recovery window. Cleared again if the value breaches
        /// before `for_seconds` of recovery have accumulated.
        below_since: Option<i64>,
    },
}

/// A state transition worth notifying about, with the observed value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertEvent {
    /// The rule started firing.
    Fired { value: f64 },
    /// The rule keeps firing and the cooldown elapsed since the last message.
    StillFiring { value: f64 },
    /// The rule stopped firing.
    Resolved { value: f64 },
}

impl AlertState {
    /// Feed one observation (at unix-seconds `now`) into the state machine.
    /// Returns the transition to notify about, if any.
    pub fn observe(&mut self, cond: &AlertCondition, value: f64, now: i64) -> Option<AlertEvent> {
        let breaching = value >= cond.threshold;
        match *self {
            AlertState::Ok if breaching => {
                if cond.for_seconds == 0 {
                    *self = AlertState::Firing {
                        since: now,
                        last_notified: now,
                        below_since: None,
                    };
                    Some(AlertEvent::Fired { value })
                } else {
                    *self = AlertState::Pending { since: now };
                    None
                }
            }
            AlertState::Ok => None,
            AlertState::Pending { since } => {
                if !breaching {
                    *self = AlertState::Ok;
                    None
                } else if now.saturating_sub(since) >= cond.for_seconds as i64 {
                    *self = AlertState::Firing {
                        since,
                        last_notified: now,
                        below_since: None,
                    };
                    Some(AlertEvent::Fired { value })
                } else {
                    None
                }
            }
            AlertState::Firing {
                since,
                last_notified,
                below_since,
            } => {
                if breaching {
                    // Any breach cancels a partial recovery window.
                    let refire = cond.cooldown_seconds > 0
                        && now.saturating_sub(last_notified) >= cond.cooldown_seconds as i64;
                    *self = AlertState::Firing {
                        since,
                        last_notified: if refire { now } else { last_notified },
                        below_since: None,
                    };
                    refire.then_some(AlertEvent::StillFiring { value })
                } else {
                    let below_since = below_since.unwrap_or(now);
                    if now.saturating_sub(below_since) >= cond.for_seconds as i64 {
                        *self = AlertState::Ok;
                        Some(AlertEvent::Resolved { value })
                    } else {
                        *self = AlertState::Firing {
                            since,
                            last_notified,
                            below_since: Some(below_since),
                        };
                        None
                    }
                }
            }
        }
    }

    pub fn is_firing(&self) -> bool {
        matches!(self, AlertState::Firing { .. })
    }
}

/// The (host, project, service) an alert event concerns, for message bodies.
pub struct AlertTarget<'a> {
    pub hostname: &'a str,
    pub project: &'a str,
    pub service: &'a str,
}

/// Build the notification for an alert event. `dashboard_url` appends a deep
/// link to the container details page (the hosted controller passes it; the
/// standalone agent has none). The email subject is keyed by (metric, service,
/// host) — not the event type — so fired/still-firing/resolved messages for
/// one rule thread into a single mail conversation.
pub fn alert_message(
    cond: &AlertCondition,
    target: &AlertTarget,
    event: &AlertEvent,
    dashboard_url: Option<&str>,
) -> Message {
    let metric = cond.metric;
    let (title, value) = match event {
        AlertEvent::Fired { value } => (
            format!(
                "Alert: {} at {} on {}",
                metric.label(),
                metric.format_value(*value),
                target.service,
            ),
            *value,
        ),
        AlertEvent::StillFiring { value } => (
            format!(
                "Still firing: {} at {} on {}",
                metric.label(),
                metric.format_value(*value),
                target.service,
            ),
            *value,
        ),
        AlertEvent::Resolved { value } => (
            format!(
                "Resolved: {} back at {} on {}",
                metric.label(),
                metric.format_value(*value),
                target.service,
            ),
            *value,
        ),
    };

    let mut body = format!(
        "{} is {} (threshold {} sustained over {})\n(project {} | service {} | host {})",
        metric.as_str(),
        metric.format_value(value),
        metric.format_value(cond.threshold),
        format_duration_secs(cond.for_seconds),
        target.project,
        target.service,
        target.hostname,
    );
    if let Some(base) = dashboard_url {
        body.push_str(&format!(
            "\n\nView details: {}/containers/{}/{}/{}",
            base.trim_end_matches('/'),
            target.hostname,
            target.project,
            target.service,
        ));
    }

    Message::new(title, body).with_subject(alert_email_subject(
        metric,
        target.service,
        target.hostname,
    ))
}

/// Stable email subject / thread key for all notifications of one metric on
/// one service+host, so fire/resolve pairs group into one conversation.
pub fn alert_email_subject(metric: AlertMetric, service: &str, hostname: &str) -> String {
    format!("Hoister alert: {} {service} on {hostname}", metric.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cond(threshold: f64, for_seconds: u64, cooldown_seconds: u64) -> AlertCondition {
        AlertCondition {
            metric: AlertMetric::CpuPct,
            threshold,
            for_seconds,
            cooldown_seconds,
        }
    }

    fn sample(cpu: f64, mem: u64, limit: u64) -> ContainerMetricSample {
        ContainerMetricSample {
            cpu_pct: cpu,
            mem_bytes: mem,
            mem_limit_bytes: limit,
            net_rx_bytes: 0,
            net_tx_bytes: 0,
            disk_read_bytes: 0,
            disk_write_bytes: 0,
        }
    }

    #[test]
    fn fires_only_after_sustained_breach() {
        let c = cond(80.0, 300, 0);
        let mut s = AlertState::default();
        assert_eq!(s.observe(&c, 90.0, 0), None, "first breach only pends");
        assert_eq!(s.observe(&c, 91.0, 120), None, "still inside the window");
        assert_eq!(
            s.observe(&c, 92.0, 300),
            Some(AlertEvent::Fired { value: 92.0 }),
            "sustained for the full window fires"
        );
    }

    #[test]
    fn short_spike_does_not_fire() {
        let c = cond(80.0, 300, 0);
        let mut s = AlertState::default();
        assert_eq!(s.observe(&c, 95.0, 0), None);
        assert_eq!(s.observe(&c, 20.0, 60), None, "recovery resets the window");
        assert_eq!(s, AlertState::Ok);
        // A later spike starts a fresh window instead of inheriting the old one.
        assert_eq!(s.observe(&c, 95.0, 240), None);
        assert_eq!(s.observe(&c, 95.0, 480), None, "only 240s into new window");
    }

    #[test]
    fn zero_for_fires_immediately() {
        let c = cond(80.0, 0, 0);
        let mut s = AlertState::default();
        assert_eq!(
            s.observe(&c, 80.0, 0),
            Some(AlertEvent::Fired { value: 80.0 }),
            "threshold is inclusive"
        );
    }

    #[test]
    fn resolves_only_after_sustained_recovery() {
        let c = cond(80.0, 120, 0);
        let mut s = AlertState::default();
        s.observe(&c, 90.0, 0);
        s.observe(&c, 90.0, 120); // fired
        assert!(s.is_firing());
        assert_eq!(s.observe(&c, 50.0, 180), None, "recovery window starts");
        assert_eq!(
            s.observe(&c, 85.0, 240),
            None,
            "breach cancels the recovery window without a new notification"
        );
        assert_eq!(s.observe(&c, 50.0, 300), None);
        assert_eq!(
            s.observe(&c, 50.0, 420),
            Some(AlertEvent::Resolved { value: 50.0 }),
            "sustained recovery resolves"
        );
        assert_eq!(s, AlertState::Ok);
    }

    #[test]
    fn cooldown_renotifies_while_firing() {
        let c = cond(80.0, 0, 600);
        let mut s = AlertState::default();
        assert!(matches!(
            s.observe(&c, 90.0, 0),
            Some(AlertEvent::Fired { .. })
        ));
        assert_eq!(s.observe(&c, 90.0, 300), None, "inside cooldown");
        assert!(matches!(
            s.observe(&c, 90.0, 600),
            Some(AlertEvent::StillFiring { .. })
        ));
        assert_eq!(s.observe(&c, 90.0, 900), None, "cooldown restarted");
    }

    #[test]
    fn zero_cooldown_notifies_once_per_episode() {
        let c = cond(80.0, 0, 0);
        let mut s = AlertState::default();
        assert!(matches!(
            s.observe(&c, 90.0, 0),
            Some(AlertEvent::Fired { .. })
        ));
        for t in 1..100 {
            assert_eq!(s.observe(&c, 90.0, t * 60), None);
        }
    }

    #[test]
    fn mem_pct_needs_a_limit() {
        let s = sample(0.0, 512, 0);
        assert_eq!(AlertMetric::MemPct.value(&s), None);
        let s = sample(0.0, 512, 1024);
        assert_eq!(AlertMetric::MemPct.value(&s), Some(50.0));
    }

    #[test]
    fn duration_parsing() {
        assert_eq!(parse_duration_secs("300"), Some(300));
        assert_eq!(parse_duration_secs("30s"), Some(30));
        assert_eq!(parse_duration_secs("5m"), Some(300));
        assert_eq!(parse_duration_secs("2h"), Some(7200));
        assert_eq!(parse_duration_secs("1d"), Some(86_400));
        assert_eq!(parse_duration_secs(""), None);
        assert_eq!(parse_duration_secs("5x"), None);
        assert_eq!(parse_duration_secs("-5m"), None);
    }

    #[test]
    fn duration_formatting() {
        assert_eq!(format_duration_secs(0), "0s");
        assert_eq!(format_duration_secs(90), "1m 30s");
        assert_eq!(format_duration_secs(3600), "1h");
        assert_eq!(format_duration_secs(300), "5m");
    }

    #[test]
    fn metric_roundtrips_through_str() {
        for m in [
            AlertMetric::CpuPct,
            AlertMetric::MemPct,
            AlertMetric::MemBytes,
        ] {
            assert_eq!(AlertMetric::parse(m.as_str()), Some(m));
        }
    }

    #[test]
    fn message_carries_threshold_and_target() {
        let c = cond(80.0, 300, 0);
        let target = AlertTarget {
            hostname: "host1",
            project: "proj",
            service: "web",
        };
        let msg = alert_message(
            &c,
            &target,
            &AlertEvent::Fired { value: 92.5 },
            Some("https://hoister.io/"),
        );
        assert_eq!(msg.title, "Alert: cpu at 92.5% on web");
        assert!(msg.body.contains("threshold 80.0% sustained over 5m"));
        assert!(
            msg.body
                .contains("https://hoister.io/containers/host1/proj/web")
        );
        assert_eq!(
            msg.subject.as_deref(),
            Some("Hoister alert: cpu_pct web on host1")
        );
    }
}
