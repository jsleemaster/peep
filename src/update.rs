use std::sync::Arc;
use tokio::sync::Mutex;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_API: &str = "https://api.github.com/repos/jsleemaster/peep/releases/latest";

#[derive(Clone)]
pub struct UpdateStatus {
    inner: Arc<Mutex<Option<String>>>,
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

    #[allow(dead_code)]
    pub async fn get(&self) -> Option<String> {
        self.inner.lock().await.clone()
    }

    pub fn try_get(&self) -> Option<String> {
        self.inner.try_lock().ok().and_then(|g| g.clone())
    }

    pub fn current() -> &'static str {
        CURRENT_VERSION
    }
}

/// Check for updates and auto-upgrade the binary before TUI starts.
/// Prints progress to stderr. Returns true if upgraded (caller should re-exec).
#[cfg(feature = "update-check")]
pub async fn auto_upgrade() -> bool {
    use std::env;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    // Skip if explicitly disabled
    if env::var("PEEP_NO_AUTO_UPDATE").is_ok() {
        return false;
    }

    let latest = match fetch_latest_version().await {
        Some(v) => v,
        None => return false,
    };

    if !is_newer(&latest, CURRENT_VERSION) {
        return false;
    }

    eprintln!(
        "peep v{} → v{} available, upgrading...",
        CURRENT_VERSION, latest
    );

    // Determine platform asset name
    let asset_name = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "peep-macos-arm64.tar.gz",
        ("macos", "x86_64") => "peep-macos-intel.tar.gz",
        ("linux", "x86_64") => "peep-linux-x86_64.tar.gz",
        ("linux", "aarch64") => "peep-linux-arm64.tar.gz",
        _ => {
            eprintln!("  unsupported platform, skipping auto-upgrade");
            return false;
        }
    };

    let download_url = format!(
        "https://github.com/jsleemaster/peep/releases/download/v{}/{}",
        latest, asset_name
    );

    // Download
    let client = match reqwest::Client::builder()
        .user_agent("peep-auto-upgrade")
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    let resp = match client.get(&download_url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => {
            eprintln!("  download failed, skipping");
            return false;
        }
    };

    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(_) => {
            eprintln!("  download failed, skipping");
            return false;
        }
    };

    // Extract tar.gz → find the binary
    let decoder = flate2::read::GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(decoder);

    let current_exe = match env::current_exe() {
        Ok(p) => p,
        Err(_) => return false,
    };

    let tmp_path = current_exe.with_extension("new");

    let entries = match archive.entries() {
        Ok(e) => e,
        Err(_) => {
            eprintln!("  failed to read archive, skipping");
            return false;
        }
    };

    let mut found = false;
    for mut entry in entries.flatten() {
        let path = entry.path().ok().map(|p| p.to_path_buf());
        if let Some(p) = path {
            if p.file_name().and_then(|n| n.to_str()) == Some("peep") {
                if entry.unpack(&tmp_path).is_ok() {
                    found = true;
                }
                break;
            }
        }
    }

    if !found {
        let _ = fs::remove_file(&tmp_path);
        eprintln!("  binary not found in archive, skipping");
        return false;
    }

    // Make executable
    if let Ok(metadata) = fs::metadata(&tmp_path) {
        let mut perms = metadata.permissions();
        perms.set_mode(0o755);
        let _ = fs::set_permissions(&tmp_path, perms);
    }

    // Atomic replace: rename old → .bak, new → current
    let bak_path = current_exe.with_extension("bak");
    let _ = fs::remove_file(&bak_path);
    if fs::rename(&current_exe, &bak_path).is_err() {
        let _ = fs::remove_file(&tmp_path);
        eprintln!("  failed to replace binary, skipping");
        return false;
    }
    if fs::rename(&tmp_path, &current_exe).is_err() {
        // Rollback
        let _ = fs::rename(&bak_path, &current_exe);
        eprintln!("  failed to replace binary, skipping");
        return false;
    }
    let _ = fs::remove_file(&bak_path);

    eprintln!("  upgraded to v{}!", latest);
    true
}

#[cfg(not(feature = "update-check"))]
pub async fn auto_upgrade() -> bool {
    false
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
        Some(tag.trim_start_matches('v').to_string())
    }

    #[cfg(not(feature = "update-check"))]
    {
        None
    }
}

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
