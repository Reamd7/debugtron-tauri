use crate::targets::types::{AppInfo, AppMetadata, TargetType};
use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub async fn discover_apps(target_id: &str) -> Result<Vec<AppInfo>> {
    println!("[WINDOWS] Starting app discovery...");

    let mut search_dirs = Vec::new();

    // Add Program Files directories
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        search_dirs.push(PathBuf::from(program_files));
    }
    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        search_dirs.push(PathBuf::from(program_files_x86));
    }

    // Add LocalAppData directory (where many Electron apps install)
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        search_dirs.push(PathBuf::from(local_app_data));
    }

    // Add user's home directory Programs folder
    if let Some(home_dir) = dirs::home_dir() {
        search_dirs.push(home_dir.join("Programs"));
    }

    let mut apps = Vec::new();

    for dir in search_dirs {
        if !dir.exists() {
            println!("[WINDOWS] Directory does not exist, skipping: {}", dir.display());
            continue;
        }

        println!("[WINDOWS] Scanning directory: {}", dir.display());

        // Walk directory tree, but limit depth to avoid scanning too deep
        for entry in WalkDir::new(&dir)
            .max_depth(3)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Check if this looks like an Electron app
            if let Some(app_info) = check_electron_app(path, target_id) {
                println!("[WINDOWS] Found Electron app: {}", app_info.name);
                apps.push(app_info);
            }
        }
    }

    println!("[WINDOWS] Discovery complete. Found {} Electron apps", apps.len());
    Ok(apps)
}

fn check_electron_app(path: &Path, target_id: &str) -> Option<AppInfo> {
    // Skip if not a directory
    if !path.is_dir() {
        return None;
    }

    // Look for resources directory (common in Electron apps)
    let resources_dir = path.join("resources");
    if !resources_dir.exists() {
        return None;
    }

    // Check for app.asar or app directory
    let app_asar = resources_dir.join("app.asar");
    let app_dir = resources_dir.join("app");

    let has_asar = app_asar.exists();
    let has_app_dir = app_dir.exists();

    if !has_asar && !has_app_dir {
        return None;
    }

    // Look for package.json
    let package_json_path = if has_app_dir && app_dir.join("package.json").exists() {
        app_dir.join("package.json")
    } else if has_asar {
        // If only app.asar exists, we can't easily read it without unpacking
        // For now, we'll try to find package.json in the parent directory
        path.join("package.json")
    } else {
        return None;
    };

    // Parse package.json
    if !package_json_path.exists() {
        // If no package.json found, use directory name as fallback
        return create_fallback_app_info(path, target_id);
    }

    parse_package_json(&package_json_path, path, target_id)
}

fn parse_package_json(package_json_path: &Path, app_path: &Path, target_id: &str) -> Option<AppInfo> {
    let content = std::fs::read_to_string(package_json_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    // Extract app name
    let name = json.get("name")
        .or_else(|| json.get("productName"))
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            app_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown App")
        })
        .to_string();

    // Find executable
    let exe_path_buf = find_executable(app_path)?;
    let exe_path = exe_path_buf.to_string_lossy().to_string();

    // Extract icon (pass exe path for icon extraction)
    let icon = extract_icon(app_path, &exe_path_buf);

    // Generate app ID
    let app_id = format!("{}:{}", target_id, sanitize_app_id(&name));

    Some(AppInfo {
        id: app_id,
        name,
        icon,
        exe_path: Some(exe_path),
        target_id: target_id.to_string(),
        target_type: TargetType::Local,
        metadata: Some(AppMetadata {
            platform: Some("windows".to_string()),
            device_info: None,
            package_name: None,
        }),
    })
}

fn create_fallback_app_info(app_path: &Path, target_id: &str) -> Option<AppInfo> {
    let name = app_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown App")
        .to_string();

    let exe_path_buf = find_executable(app_path)?;
    let exe_path = exe_path_buf.to_string_lossy().to_string();
    let icon = extract_icon(app_path, &exe_path_buf);
    let app_id = format!("{}:{}", target_id, sanitize_app_id(&name));

    Some(AppInfo {
        id: app_id,
        name,
        icon,
        exe_path: Some(exe_path),
        target_id: target_id.to_string(),
        target_type: TargetType::Local,
        metadata: Some(AppMetadata {
            platform: Some("windows".to_string()),
            device_info: None,
            package_name: None,
        }),
    })
}

fn find_executable(app_path: &Path) -> Option<PathBuf> {
    // First, try to find any .exe file in the app directory
    if let Ok(entries) = std::fs::read_dir(app_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("exe") {
                let file_name = path.file_name()?.to_str()?;

                // Prefer non-Update.exe files
                if !file_name.contains("Update") && !file_name.contains("unins") {
                    return Some(path);
                }
            }
        }
    }

    None
}

fn extract_icon(app_path: &Path, exe_path: &Path) -> String {
    println!("[WINDOWS] Extracting icon for: {}", exe_path.display());

    // Strategy 1: Try standalone icon files first (faster and more reliable)
    let icon_paths = vec![
        app_path.join("resources").join("app.ico"),
        app_path.join("resources").join("icon.ico"),
        app_path.join("resources").join("app").join("icon.ico"),
        app_path.join("resources").join("app").join("icon.png"),
        app_path.join("icon.ico"),
        app_path.join("app.ico"),
    ];

    for icon_path in &icon_paths {
        if let Some(icon_data) = read_icon_file(icon_path) {
            println!("[WINDOWS] Found icon file at: {}", icon_path.display());
            return icon_data;
        }
    }

    // Strategy 2: Search recursively in resources directory
    if let Some(icon_data) = search_icon_in_resources(app_path) {
        return icon_data;
    }

    // Strategy 3: Try to extract from .exe file (slower, may fail)
    println!("[WINDOWS] Trying to extract icon from exe file...");
    if let Some(icon_data) = extract_icon_from_exe(exe_path) {
        println!("[WINDOWS] Successfully extracted icon from exe");
        return icon_data;
    }

    println!("[WINDOWS] No icon found for: {}", exe_path.display());
    String::new()
}

fn read_icon_file(path: &Path) -> Option<String> {
    use std::io::Read;

    if !path.exists() {
        return None;
    }

    let ext = path.extension()?.to_str()?;
    let mime_type = match ext {
        "ico" => "image/x-icon",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => return None,
    };

    let mut file = std::fs::File::open(path).ok()?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).ok()?;

    let base64_data = base64_encode(&buffer);
    Some(format!("data:{};base64,{}", mime_type, base64_data))
}

fn search_icon_in_resources(app_path: &Path) -> Option<String> {
    let resources_dir = app_path.join("resources");
    if !resources_dir.exists() {
        return None;
    }

    // Search for icon files (ico, png, jpg) in resources directory
    for entry in WalkDir::new(&resources_dir)
        .max_depth(3)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            let file_name = path.file_stem()?.to_str()?.to_lowercase();

            // Look for files named "icon", "app", or containing "icon"
            if (ext == "ico" || ext == "png") &&
               (file_name.contains("icon") || file_name == "app") {
                if let Some(icon_data) = read_icon_file(path) {
                    println!("[WINDOWS] Found icon at: {}", path.display());
                    return Some(icon_data);
                }
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn extract_icon_from_exe(exe_path: &Path) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::shellapi::{ExtractIconW, SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use winapi::um::winuser::{GetIconInfo, DestroyIcon, DrawIconEx, ICONINFO, GetDC, ReleaseDC};
    use winapi::um::wingdi::{
        CreateCompatibleDC, SelectObject, DeleteDC,
        DeleteObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        CreateDIBSection
    };
    use std::ptr::null_mut;
    use std::mem::size_of;

    unsafe {
        // Convert path to wide string
        let wide_path: Vec<u16> = exe_path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // Try to get file icon using SHGetFileInfoW
        let mut shfi: SHFILEINFOW = std::mem::zeroed();
        let result = SHGetFileInfoW(
            wide_path.as_ptr(),
            0,
            &mut shfi as *mut _,
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );

        let hicon = if result != 0 {
            shfi.hIcon
        } else {
            // Fallback to ExtractIconW
            let icon = ExtractIconW(null_mut(), wide_path.as_ptr(), 0);
            if icon.is_null() || icon as isize <= 1 {
                println!("[WINDOWS] Failed to extract icon from: {}", exe_path.display());
                return None;
            }
            icon
        };

        if hicon.is_null() {
            println!("[WINDOWS] Null icon handle for: {}", exe_path.display());
            return None;
        }

        // Get icon info
        let mut icon_info: ICONINFO = std::mem::zeroed();
        if GetIconInfo(hicon, &mut icon_info) == 0 {
            println!("[WINDOWS] Failed to get icon info");
            DestroyIcon(hicon);
            return None;
        }

        // Get screen DC
        let screen_dc = GetDC(null_mut());
        if screen_dc.is_null() {
            DeleteObject(icon_info.hbmColor as *mut _);
            DeleteObject(icon_info.hbmMask as *mut _);
            DestroyIcon(hicon);
            return None;
        }

        // Create a compatible DC
        let hdc = CreateCompatibleDC(screen_dc);
        if hdc.is_null() {
            ReleaseDC(null_mut(), screen_dc);
            DeleteObject(icon_info.hbmColor as *mut _);
            DeleteObject(icon_info.hbmMask as *mut _);
            DestroyIcon(hicon);
            return None;
        }

        // Icon dimensions
        let icon_size = 32i32;

        // Create bitmap info for DIB section
        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = icon_size;
        bmi.bmiHeader.biHeight = -icon_size; // Top-down DIB
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        let mut bits: *mut winapi::ctypes::c_void = null_mut();
        let hbmp = CreateDIBSection(
            hdc,
            &bmi as *const _,
            DIB_RGB_COLORS,
            &mut bits,
            null_mut(),
            0,
        );

        if hbmp.is_null() || bits.is_null() {
            println!("[WINDOWS] Failed to create DIB section");
            DeleteDC(hdc);
            ReleaseDC(null_mut(), screen_dc);
            DeleteObject(icon_info.hbmColor as *mut _);
            DeleteObject(icon_info.hbmMask as *mut _);
            DestroyIcon(hicon);
            return None;
        }

        let old_bmp = SelectObject(hdc, hbmp as *mut _);

        // Draw icon to bitmap (DI_NORMAL = 0x0003)
        let draw_result = DrawIconEx(hdc, 0, 0, hicon, icon_size, icon_size, 0, null_mut(), 0x0003);

        if draw_result == 0 {
            println!("[WINDOWS] Failed to draw icon");
        }

        // Copy bitmap data
        let buffer_size = (icon_size * icon_size * 4) as usize;
        let mut buffer: Vec<u8> = vec![0; buffer_size];
        std::ptr::copy_nonoverlapping(bits as *const u8, buffer.as_mut_ptr(), buffer_size);

        // Clean up
        SelectObject(hdc, old_bmp);
        DeleteObject(hbmp as *mut _);
        DeleteDC(hdc);
        ReleaseDC(null_mut(), screen_dc);
        DeleteObject(icon_info.hbmColor as *mut _);
        DeleteObject(icon_info.hbmMask as *mut _);
        DestroyIcon(hicon);

        // Convert BGRA to RGBA (Windows bitmaps are in BGRA format)
        for i in (0..buffer.len()).step_by(4) {
            buffer.swap(i, i + 2); // Swap B and R
        }

        // Try to encode as PNG using image crate
        println!("[WINDOWS] Buffer size: {}, Icon size: {}", buffer.len(), icon_size);
        match encode_to_png(&buffer, icon_size as u32) {
            Some(png_data) => {
                println!("[WINDOWS] PNG data size: {} bytes", png_data.len());
                let base64_data = base64_encode(&png_data);
                println!("[WINDOWS] Base64 data length: {} chars", base64_data.len());
                println!("[WINDOWS] Successfully extracted and encoded icon from: {}", exe_path.display());
                Some(format!("data:image/png;base64,{}", base64_data))
            }
            None => {
                println!("[WINDOWS] Failed to encode PNG for: {}", exe_path.display());
                None
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn encode_to_png(rgba_data: &[u8], size: u32) -> Option<Vec<u8>> {
    use image::{ImageBuffer, Rgba, codecs::png::PngEncoder, ImageEncoder};
    use std::io::Cursor;

    // Create image buffer from RGBA data
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(size, size, rgba_data.to_vec())?;

    // Encode to PNG
    let mut png_data = Vec::new();
    let mut cursor = Cursor::new(&mut png_data);

    let encoder = PngEncoder::new(&mut cursor);
    if encoder.write_image(&img, size, size, image::ExtendedColorType::Rgba8).is_ok() {
        Some(png_data)
    } else {
        None
    }
}

#[cfg(not(target_os = "windows"))]
fn extract_icon_from_exe(_exe_path: &Path) -> Option<String> {
    None
}

fn sanitize_app_id(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase()
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
