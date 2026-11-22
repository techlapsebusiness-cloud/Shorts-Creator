//! macOS Finder desktop-icon visibility. Best effort: Finder restarts when this runs.

use std::process::Command;

pub fn set_desktop_icons_visible(visible: bool) -> Result<(), String> {
    let value = if visible { "true" } else { "false" };
    let status = Command::new("defaults")
        .args(["write", "com.apple.finder", "CreateDesktop", "-bool", value])
        .status()
        .map_err(|error| format!("defaults: {error}"))?;
    if !status.success() {
        return Err(format!("defaults exited {status}"));
    }
    let status = Command::new("killall")
        .arg("Finder")
        .status()
        .map_err(|error| format!("killall Finder: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("killall Finder exited {status}"))
    }
}
