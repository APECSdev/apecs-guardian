// src/comm/api.rs

/*!
 * Api — the daemon's only network surface. It POSTs JSON to the APECS control
 * plane (`/v1/guardian/`). All requests are authenticated with the node's
 * `x-apecs-key`. There is no inbound listener; the daemon polls outbound only.
 */

/// Default control-plane base URL.
pub const DEFAULT_BASE: &str = "https://apecs.dev/v1/guardian/";

/// Build the full URL for an endpoint under a base.
pub fn build_url(base: &str, endpoint: &str) -> String {
    let base = base.trim_end_matches('/');
    let endpoint = endpoint.trim_start_matches('/');
    format!("{}/{}", base, endpoint)
}

/// POST a JSON body to an endpoint, authenticated with the node's key.
///
/// Returns the response body on success. TLS is rustls (no OpenSSL), so the
/// binary stays statically linkable and cross-compiles cleanly to Windows.
pub async fn post(
    base: &str,
    endpoint: &str,
    json: &str,
    api_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = build_url(base, endpoint);

    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("x-apecs-key", api_key)
        .body(json.to_string())
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(format!("api error: HTTP {}", resp.status()).into());
    }

    Ok(resp.text().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_url_with_trailing_slash_handling() {
        assert_eq!(
            build_url("https://apecs.dev/v1/guardian/", "register"),
            "https://apecs.dev/v1/guardian/register"
        );
        assert_eq!(
            build_url("https://apecs.dev/v1/guardian", "/register"),
            "https://apecs.dev/v1/guardian/register"
        );
    }
}
