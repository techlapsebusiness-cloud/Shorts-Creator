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

/// Rough budget for screen (high bitrate) + cam + mic WAV.
pub const BYTES_PER_MINUTE: u64 = 180_000_000;

pub fn estimate(path: &Path) -> Result<DiskEstimate, String> {
    let free_bytes = free_space(path)?;
    let minutes_remaining = free_bytes as f64 / BYTES_PER_MINUTE as f64;
    Ok(DiskEstimate {
        free_bytes,
        bytes_per_minute: BYTES_PER_MINUTE,
        minutes_remaining,
    })
}

fn free_space(path: &Path) -> Result<u64, String> {
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
