use crate::config::EcrRegistry;
use base64::Engine as _;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn derive_signing_key(secret: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
    let k_date = hmac_sha256(format!("AWS4{secret}").as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, service.as_bytes());
    hmac_sha256(&k_service, b"aws4_request")
}

/// Fetches a short-lived ECR authorization token using AWS Signature V4 signing.
/// Returns `(username, password)` suitable for use as Docker registry credentials.
pub(crate) async fn get_ecr_token(
    client: &reqwest::Client,
    ecr: &EcrRegistry,
) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    let region = &ecr.region;
    let host = format!("ecr.{region}.amazonaws.com");
    let endpoint = format!("https://{host}/");
    let service = "ecr";
    let body = "{}";
    let target = "AmazonEC2ContainerRegistry_V20150921.GetAuthorizationToken";

    let now = chrono::Utc::now();
    let date_str = now.format("%Y%m%d").to_string();
    let datetime_str = now.format("%Y%m%dT%H%M%SZ").to_string();

    let payload_hash = sha256_hex(body.as_bytes());

    // Canonical headers must be sorted alphabetically and each end with \n.
    let canonical_headers = format!(
        "content-type:application/x-amz-json-1.1\nhost:{host}\nx-amz-date:{datetime_str}\nx-amz-target:{target}\n"
    );
    let signed_headers = "content-type;host;x-amz-date;x-amz-target";

    // Format: method\nuri\nquerystring\ncanonical_headers\nsigned_headers\npayload_hash
    // canonical_headers already ends with \n, so the extra \n creates the required blank line.
    let canonical_request =
        format!("POST\n/\n\n{canonical_headers}\n{signed_headers}\n{payload_hash}");

    let credential_scope = format!("{date_str}/{region}/{service}/aws4_request");
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{datetime_str}\n{credential_scope}\n{}",
        sha256_hex(canonical_request.as_bytes())
    );

    let signing_key = derive_signing_key(&ecr.secret_access_key, &date_str, region, service);
    let signature: String = hmac_sha256(&signing_key, string_to_sign.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}",
        ecr.access_key_id
    );

    let response = client
        .post(&endpoint)
        .header("content-type", "application/x-amz-json-1.1")
        .header("x-amz-date", &datetime_str)
        .header("x-amz-target", target)
        .header("authorization", &authorization)
        .body(body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("ECR API returned {status}: {text}").into());
    }

    let json: serde_json::Value = response.json().await?;

    let auth_token = json["authorizationData"][0]["authorizationToken"]
        .as_str()
        .ok_or("Missing authorizationToken in ECR response")?;

    let decoded = base64::engine::general_purpose::STANDARD.decode(auth_token)?;
    let credentials = String::from_utf8(decoded)?;

    credentials
        .split_once(':')
        .map(|(u, p)| (u.to_string(), p.to_string()))
        .ok_or_else(|| "Invalid ECR credentials format".into())
}
