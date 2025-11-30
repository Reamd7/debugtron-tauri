use crate::targets::types::{AppInfo, AppMetadata, TargetType};
use anyhow::Result;

pub async fn discover_apps(target_id: &str) -> Result<Vec<AppInfo>> {
    // TODO: Implement Linux app discovery
    // - Scan /usr/share/applications and ~/.local/share/applications
    // - Parse .desktop files
    // - Detect Electron apps by checking executable dependencies (ldd)

    eprintln!("Linux app discovery not yet implemented");
    Ok(vec![])
}

#[allow(dead_code)]
fn parse_desktop_file(_path: &std::path::Path) -> Result<Option<AppInfo>> {
    // TODO: Parse .desktop file format
    Ok(None)
}
