//! Hostname validation for user-supplied notifier targets.
//!
//! The notifier dispatch path makes outbound HTTP/SMTP requests to URLs
//! the tenant chose. On the hosted controller this runs inside the docker
//! bridge with route to `backend-db`, `hoister-controller:3034`, and (on
//! some clouds) the metadata service at `169.254.169.254`. Without
//! validation a free-tier signup is a one-click SSRF probe.
//!
//! We validate at *create* time:
//!   - Slack: webhook URL must be `https://hooks.slack.com/…`.
//!   - Discord webhook: URL must be `https://discord.com/api/webhooks/…`.
//!   - Gotify: server must be `https://`, hostname must resolve to a
//!     public IP (no loopback, RFC1918, link-local, ULA, etc.).
//!   - Email: recipient must be a syntactically valid address. Delivery is
//!     via the controller-wide Resend account, so there is no user-supplied
//!     host to SSRF-check.
//!   - Telegram / Discord: kind-fixed to api.telegram.org / discord.com
//!     inside chatterbox; no user-supplied host to check.
//!
//! Re-validation at dispatch time (DNS rebinding) is deliberately not
//! done — would double dispatch latency, and the attacker would need
//! control of the resolver dockerd uses. Tracked as a follow-up.

use crate::domain::notifiers::models::NotifierConfig;
use std::net::IpAddr;
use tokio::net::lookup_host;

#[derive(Debug)]
pub enum ValidationError {
    Empty(&'static str),
    NotHttps,
    UnsupportedDomain(&'static str),
    InvalidUrl,
    DnsLookupFailed,
    PrivateAddress,
}

impl ValidationError {
    /// Human-readable message safe to echo to the tenant.
    pub fn user_message(&self) -> String {
        match self {
            ValidationError::Empty(field) => format!("{field} is required"),
            ValidationError::NotHttps => "URL must use https://".to_string(),
            ValidationError::UnsupportedDomain(m) => (*m).to_string(),
            ValidationError::InvalidUrl => "URL could not be parsed".to_string(),
            ValidationError::DnsLookupFailed => "Hostname did not resolve".to_string(),
            ValidationError::PrivateAddress => {
                "Hostname resolves to a private or loopback address".to_string()
            }
        }
    }
}

pub async fn validate_config(config: &NotifierConfig) -> Result<(), ValidationError> {
    match config {
        NotifierConfig::Slack(c) => validate_slack(&c.webhook),
        NotifierConfig::Gotify(c) => validate_gotify(&c.server).await,
        NotifierConfig::Email(c) => validate_email_recipient(&c.recipient),
        NotifierConfig::DiscordWebhook(c) => validate_discord_webhook(&c.webhook),
        NotifierConfig::Telegram(_) | NotifierConfig::Discord(_) => Ok(()),
    }
}

/// Discord incoming webhooks are always served from `discord.com`, so we
/// pin the host the same way we pin Slack — no user-supplied host means no
/// SSRF surface, and we can skip the DNS/private-IP dance.
fn validate_discord_webhook(webhook: &str) -> Result<(), ValidationError> {
    if webhook.is_empty() {
        return Err(ValidationError::Empty("Discord webhook URL"));
    }
    if !webhook.starts_with("https://discord.com/api/webhooks/")
        && !webhook.starts_with("https://discordapp.com/api/webhooks/")
    {
        return Err(ValidationError::UnsupportedDomain(
            "Discord webhook must be on https://discord.com/api/webhooks/",
        ));
    }
    Ok(())
}

fn validate_slack(webhook: &str) -> Result<(), ValidationError> {
    if webhook.is_empty() {
        return Err(ValidationError::Empty("Slack webhook URL"));
    }
    if !webhook.starts_with("https://hooks.slack.com/") {
        return Err(ValidationError::UnsupportedDomain(
            "Slack webhook must be on https://hooks.slack.com/",
        ));
    }
    Ok(())
}

async fn validate_gotify(server: &str) -> Result<(), ValidationError> {
    if server.is_empty() {
        return Err(ValidationError::Empty("Gotify server URL"));
    }
    let url = url::Url::parse(server).map_err(|_| ValidationError::InvalidUrl)?;
    if url.scheme() != "https" {
        return Err(ValidationError::NotHttps);
    }
    let host = url.host_str().ok_or(ValidationError::InvalidUrl)?;
    let port = url.port_or_known_default().unwrap_or(443);
    ensure_public_host(host, port).await
}

/// Email notifiers deliver through the controller-wide Resend account, so
/// there is no user-supplied host to SSRF-check — we only confirm the
/// recipient is a plausible address. Resend does the authoritative
/// validation at send time.
fn validate_email_recipient(recipient: &str) -> Result<(), ValidationError> {
    if recipient.is_empty() {
        return Err(ValidationError::Empty("Recipient email"));
    }
    let parts: Vec<&str> = recipient.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || !parts[1].contains('.') {
        return Err(ValidationError::UnsupportedDomain(
            "Recipient must be a valid email address",
        ));
    }
    Ok(())
}

async fn ensure_public_host(host: &str, port: u16) -> Result<(), ValidationError> {
    // Reject IP-literal hosts that are private even before DNS — saves a
    // lookup and avoids subtle bugs in the OS resolver returning IP
    // literals verbatim.
    if let Ok(literal) = host.parse::<IpAddr>()
        && is_disallowed_ip(literal)
    {
        return Err(ValidationError::PrivateAddress);
    }

    let addrs = lookup_host((host, port))
        .await
        .map_err(|_| ValidationError::DnsLookupFailed)?;
    let mut seen_any = false;
    for addr in addrs {
        seen_any = true;
        if is_disallowed_ip(addr.ip()) {
            return Err(ValidationError::PrivateAddress);
        }
    }
    if !seen_any {
        return Err(ValidationError::DnsLookupFailed);
    }
    Ok(())
}

fn is_disallowed_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.is_unspecified()
                || v4.octets()[0] == 0
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // fe80::/10 link-local
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7 ULA
                || matches!(v6.to_ipv4_mapped(), Some(v4) if is_disallowed_ip(IpAddr::V4(v4)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn disallows_loopback_and_private_v4() {
        for ip in [
            "127.0.0.1",
            "10.0.0.1",
            "172.16.0.1",
            "192.168.1.1",
            "169.254.169.254",
            "0.0.0.0",
            "255.255.255.255",
        ] {
            let parsed: Ipv4Addr = ip.parse().unwrap();
            assert!(
                is_disallowed_ip(IpAddr::V4(parsed)),
                "expected {ip} blocked"
            );
        }
    }

    #[test]
    fn allows_public_v4() {
        for ip in ["8.8.8.8", "1.1.1.1", "44.205.0.1"] {
            let parsed: Ipv4Addr = ip.parse().unwrap();
            assert!(
                !is_disallowed_ip(IpAddr::V4(parsed)),
                "expected {ip} allowed"
            );
        }
    }

    #[test]
    fn disallows_loopback_and_link_local_v6() {
        for ip in ["::1", "fe80::1", "fd00::1"] {
            let parsed: Ipv6Addr = ip.parse().unwrap();
            assert!(
                is_disallowed_ip(IpAddr::V6(parsed)),
                "expected {ip} blocked"
            );
        }
    }

    #[test]
    fn ipv4_mapped_inherits_v4_block() {
        let mapped: Ipv6Addr = "::ffff:127.0.0.1".parse().unwrap();
        assert!(is_disallowed_ip(IpAddr::V6(mapped)));
    }

    #[test]
    fn slack_requires_canonical_host() {
        assert!(validate_slack("https://hooks.slack.com/services/T/B/X").is_ok());
        assert!(validate_slack("http://hooks.slack.com/services/T/B/X").is_err());
        assert!(validate_slack("https://evil.example.com/").is_err());
        assert!(validate_slack("").is_err());
    }

    #[test]
    fn discord_webhook_requires_canonical_host() {
        assert!(validate_discord_webhook("https://discord.com/api/webhooks/123/abc").is_ok());
        assert!(validate_discord_webhook("https://discordapp.com/api/webhooks/123/abc").is_ok());
        assert!(validate_discord_webhook("http://discord.com/api/webhooks/123/abc").is_err());
        assert!(validate_discord_webhook("https://evil.example.com/api/webhooks/1/2").is_err());
        assert!(validate_discord_webhook("").is_err());
    }
}
