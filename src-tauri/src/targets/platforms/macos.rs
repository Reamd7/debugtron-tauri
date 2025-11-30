use crate::targets::types::{AppInfo, AppMetadata, TargetType};
use anyhow::Result;
use std::path::PathBuf;

pub async fn discover_apps(target_id: &str) -> Result<Vec<AppInfo>> {
    println!("[MACOS] Starting app discovery...");
    let app_dirs = vec![
        PathBuf::from("/Applications"),
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot get home directory"))?
            .join("Applications"),
    ];

    let mut apps = Vec::new();

    for dir in app_dirs {
        println!("[MACOS] Scanning directory: {}", dir.display());
        if !dir.exists() {
            println!("[MACOS] Directory does not exist, skipping");
            continue;
        }

        let entries = std::fs::read_dir(&dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Only process .app bundles
            if !path.extension().map_or(false, |e| e == "app") {
                continue;
            }

            // Check for Info.plist
            let info_plist_path = path.join("Contents/Info.plist");
            if !info_plist_path.exists() {
                continue;
            }

            // Parse plist
            match parse_app_bundle(&path, &info_plist_path, target_id) {
                Ok(Some(app_info)) => {
                    println!("[MACOS] Found Electron app: {}", app_info.name);
                    apps.push(app_info);
                }
                Ok(None) => continue, // Not an Electron app
                Err(e) => {
                    eprintln!("[MACOS] Error parsing {}: {}", path.display(), e);
                    continue;
                }
            }
        }
    }

    println!("[MACOS] Discovery complete. Found {} Electron apps", apps.len());
    Ok(apps)
}

fn parse_app_bundle(
    app_path: &PathBuf,
    plist_path: &PathBuf,
    target_id: &str,
) -> Result<Option<AppInfo>> {
    use plist::Value;

    // Check if it's an Electron app by looking for Electron Framework
    let electron_framework_path = app_path.join("Contents/Frameworks/Electron Framework.framework");
    if !electron_framework_path.exists() {
        return Ok(None);
    }

    let plist_data = std::fs::read(plist_path)?;
    let plist: Value = plist::from_bytes(&plist_data)?;

    let dict = plist
        .as_dictionary()
        .ok_or_else(|| anyhow::anyhow!("Invalid plist format"))?;

    // Extract bundle identifier
    let bundle_id = dict
        .get("CFBundleIdentifier")
        .and_then(|v| v.as_string())
        .unwrap_or("unknown");

    // Extract display name
    let display_name = dict
        .get("CFBundleDisplayName")
        .or_else(|| dict.get("CFBundleName"))
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| {
            app_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown App")
        });

    // Extract executable name
    let executable = dict
        .get("CFBundleExecutable")
        .and_then(|v| v.as_string())
        .ok_or_else(|| anyhow::anyhow!("No CFBundleExecutable"))?;

    let exe_path = app_path
        .join("Contents/MacOS")
        .join(executable)
        .to_string_lossy()
        .to_string();

    // Extract icon
    let icon_file = dict
        .get("CFBundleIconFile")
        .and_then(|v| v.as_string());

    let icon = if let Some(icon_name) = icon_file {
        extract_icon(app_path, icon_name)?
    } else {
        String::new()
    };

    Ok(Some(AppInfo {
        id: format!("{}:{}", target_id, bundle_id),
        name: display_name.to_string(),
        icon,
        exe_path: Some(exe_path),
        target_id: target_id.to_string(),
        target_type: TargetType::Local,
        metadata: Some(AppMetadata {
            platform: Some("macos".to_string()),
            device_info: None,
            package_name: None,
        }),
    }))
}

fn extract_icon(app_path: &PathBuf, icon_name: &str) -> Result<String> {
    use std::io::Read;

    // Construct the full path to the icon file
    let icon_path = if icon_name.ends_with(".icns") {
        app_path.join("Contents/Resources").join(icon_name)
    } else {
        app_path
            .join("Contents/Resources")
            .join(format!("{}.icns", icon_name))
    };

    if !icon_path.exists() {
        return Ok(String::new());
    }

    // Read the ICNS file
    let mut file = std::fs::File::open(&icon_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Parse ICNS format
    if buffer.len() < 8 {
        return Ok(String::new());
    }

    // Check magic number "icns"
    if &buffer[0..4] != b"icns" {
        return Ok(String::new());
    }

    // Read total size
    let total_size = u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]) as usize;
    if total_size < 8 || total_size > buffer.len() {
        return Ok(String::new());
    }

    // Parse icon entries
    let mut icons = Vec::new();
    let mut pos = 8;

    while pos + 8 <= total_size {
        let icon_type = &buffer[pos..pos + 4];
        let icon_size = u32::from_be_bytes([
            buffer[pos + 4],
            buffer[pos + 5],
            buffer[pos + 6],
            buffer[pos + 7],
        ]) as usize;

        if pos + icon_size > total_size {
            break;
        }

        let icon_data = &buffer[pos + 8..pos + icon_size];
        icons.push((icon_type.to_vec(), icon_size, icon_data.to_vec()));

        pos += icon_size;
    }

    // Sort by size (descending) and find the first PNG image
    icons.sort_by(|a, b| b.1.cmp(&a.1));

    for (_type, _size, data) in icons {
        // Check if it's a PNG image (PNG signature: 89 50 4E 47)
        if data.len() > 4 && data[1..4] == [0x50, 0x4E, 0x47] {
            // Convert to base64
            let base64_data = base64_encode(&data);
            return Ok(format!("data:image/png;base64,{}", base64_data));
        }
    }

    Ok(String::new())
}

fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    let mut i = 0;
    while i < data.len() {
        let b1 = data[i];
        let b2 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b3 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        result.push(CHARSET[(b1 >> 2) as usize] as char);
        result.push(CHARSET[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);

        if i + 1 < data.len() {
            result.push(CHARSET[(((b2 & 0x0F) << 2) | (b3 >> 6)) as usize] as char);
        } else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(CHARSET[(b3 & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }

    result
}
