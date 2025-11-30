pub mod registry;
pub mod types;
pub mod local;

#[cfg(target_os = "macos")]
pub mod platforms {
    pub mod macos;
}

#[cfg(target_os = "windows")]
pub mod platforms {
    pub mod windows;
}

#[cfg(target_os = "linux")]
pub mod platforms {
    pub mod linux;
}
