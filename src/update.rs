use std::sync::Arc;
use tokio::sync::Mutex;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_API: &str = "https://api.github.com/repos/jsleemaster/peep/releases/latest";

#[derive(Clone)]
pub struct UpdateStatus {
    inner: Arc<Mutex<Option<String>>>, // Some("0.3.0") if new version available
}

impl UpdateStatus {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    /// Spawn a background task to check for updates. Non-blocking.
    pub fn check_in_background(&self) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            if let Some(latest) = fetch_latest_version().await {
                if is_newer(&latest, CURRENT_VERSION) {
                    let mut lock = inner.lock().await;
                    *lock = Some(latest);
                }
            }
        });
    }

    /// Get the new version if available (non-blocking).
    #[allow(dead_code)]
    pub async fn get(&self) -> Option<String> {
        self.inner.lock().await.clone()
    }

    /// Try to get without blocking (for sync tick handler).
    pub fn try_get(&self) -> Option<String> {
        self.inner.try_lock().ok().and_then(|g| g.clone())
    }

    pub fn current() -> &'static str {
        CURRENT_VERSION
    }
}

async fn fetch_latest_version() -> Option<String> {
    #[cfg(feature = "update-check")]
    {
        let client = reqwest::Client::builder()
            .user_agent("peep-update-check")
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .ok()?;

        let resp = client.get(GITHUB_API).send().await.ok()?;
        if !resp.status().is_success() {
            return None;
        }

        let json: serde_json::Value = resp.json().await.ok()?;
        let tag = json.get("tag_name")?.as_str()?;
        // Strip leading 'v'
        Some(tag.trim_start_matches('v').to_string())
    }

    #[cfg(not(feature = "update-check"))]
    {
        None
    }
}

/// Simple semver comparison: is `latest` > `current`?
fn is_newer(latest: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> {
        s.split('.')
            .filter_map(|p| p.parse().ok())
            .collect()
    };
    let l = parse(latest);
    let c = parse(current);
    l > c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
    }
}
