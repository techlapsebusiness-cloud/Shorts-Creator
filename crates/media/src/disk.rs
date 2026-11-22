//! Disk-space estimates for a take directory.

use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskEstimate {
    pub free_bytes: u64,
    pub bytes_per_minute: u64,
    pub minutes_remaining: f64,
}

/// Screen 20 Mbps + camera 8 Mbps + 48 kHz mono 16-bit PCM.
pub const BYTES_PER_MINUTE: u64 = 20_000_000 / 8 * 60 + 8_000_000 / 8 * 60 + 48_000 * 2 * 60;

pub fn estimate(path: &Path) -> Result<DiskEstimate, String> {
    let free_bytes = free_bytes(path)?;
    let minutes_remaining = free_bytes as f64 / BYTES_PER_MINUTE as f64;
    Ok(DiskEstimate {
        free_bytes,
        bytes_per_minute: BYTES_PER_MINUTE,
        minutes_remaining,
    })
}

pub(crate) fn free_bytes(path: &Path) -> Result<u64, String> {
    let probe = if path.exists() {
        path.to_path_buf()
    } else if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        parent.to_path_buf()
    } else {
        Path::new(".").to_path_buf()
    };

    let output = Command::new("df")
        .args(["-k", "-P"])
        .arg(&probe)
        .output()
        .map_err(|error| format!("df failed: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "df exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout
        .lines()
        .nth(1)
        .ok_or_else(|| format!("df returned no data for {}", probe.display()))?;
    let available_kib = line
        .split_whitespace()
        .nth(3)
        .ok_or_else(|| format!("df line missing available column: {line}"))?
        .parse::<u64>()
        .map_err(|error| format!("df available parse failed: {error}"))?;
    Ok(available_kib.saturating_mul(1024))
}

pub(crate) fn directory_bytes(path: &Path) -> u64 {
    fn walk(path: &Path, total: &mut u64) {
        let Ok(entries) = std::fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.is_dir() {
                walk(&entry.path(), total);
            } else {
                *total = total.saturating_add(meta.len());
            }
        }
    }
    let mut total = 0;
    walk(path, &mut total);
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimates_workspace_disk() {
        let estimate = estimate(Path::new(".")).unwrap();
        assert!(estimate.free_bytes > 0);
        assert_eq!(estimate.bytes_per_minute, BYTES_PER_MINUTE);
        assert!(estimate.minutes_remaining > 0.0);
    }
}
