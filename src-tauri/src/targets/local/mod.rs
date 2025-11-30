use super::types::*;
use anyhow::Result;
use async_trait::async_trait;
use tauri::Manager;

pub struct LocalTargetAdapter {
    id: String,
    name: String,
    platform: String,
}

impl LocalTargetAdapter {
    pub fn new() -> Self {
        let platform = std::env::consts::OS;
        Self {
            id: format!("local-{}", platform),
            name: format!("Local ({})", Self::platform_name(platform)),
            platform: platform.to_string(),
        }
    }

    fn platform_name(os: &str) -> &str {
        match os {
            "macos" => "macOS",
            "windows" => "Windows",
            "linux" => "Linux",
            _ => os,
        }
    }
}

#[async_trait]
impl TargetAdapter for LocalTargetAdapter {
    fn get_type(&self) -> TargetType {
        TargetType::Local
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    async fn discover_apps(&self) -> Result<Vec<AppInfo>> {
        // Platform-specific implementation will be added
        #[cfg(target_os = "macos")]
        {
            crate::targets::platforms::macos::discover_apps(&self.id).await
        }

        #[cfg(target_os = "windows")]
        {
            crate::targets::platforms::windows::discover_apps(&self.id).await
        }

        #[cfg(target_os = "linux")]
        {
            crate::targets::platforms::linux::discover_apps(&self.id).await
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err(anyhow::anyhow!("Unsupported platform"))
        }
    }

    async fn launch(&self, app: &AppInfo, options: LaunchOptions) -> Result<DebugConnection> {
        use tokio::process::Command;

        let exe_path = app
            .exe_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No executable path"))?;

        // Allocate debug ports
        let node_port = portpicker::pick_unused_port()
            .ok_or_else(|| anyhow::anyhow!("No available port for node inspector"))?;
        let renderer_port = portpicker::pick_unused_port()
            .ok_or_else(|| anyhow::anyhow!("No available port for renderer"))?;

        // Prepare debug flags
        let debug_flags = options.debug_flags.unwrap_or_else(|| {
            let inspect_flag = if options.inspect_brk.unwrap_or(false) {
                format!("--inspect-brk={}", node_port)
            } else {
                format!("--inspect={}", node_port)
            };

            vec![
                inspect_flag,
                format!("--remote-debugging-port={}", renderer_port),
                // Allow all origins for DevTools (since we're debugging local apps)
                "--remote-allow-origins=*".to_string(),
            ]
        });

        // Spawn the process
        let mut command = Command::new(exe_path);
        command.args(&debug_flags);

        if let Some(cwd) = &options.cwd {
            command.current_dir(cwd);
        }

        if let Some(env) = &options.env {
            command.envs(env);
        }

        let mut child = command
            .stdin(std::process::Stdio::null())  // Don't pipe stdin to avoid EPIPE
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let pid = child.id();
        let connection_id = uuid::Uuid::new_v4().to_string();
        let app_id = app.id.clone();

        // Get app_handle from options (will be added)
        let app_handle = options.app_handle.clone();

        // Spawn task to read stdout/stderr and emit to frontend
        if let Some(stdout) = child.stdout.take() {
            let conn_id = connection_id.clone();
            let handle = app_handle.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    println!("[APP STDOUT] {}", line);
                    if let Some(h) = &handle {
                        let log_data = serde_json::json!({
                            "sessionId": conn_id,
                            "connectionId": conn_id,
                            "type": "stdout",
                            "message": line
                        });
                        let _ = h.emit_all("session-log", &log_data);
                    }
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let conn_id = connection_id.clone();
            let handle = app_handle.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    println!("[APP STDERR] {}", line);
                    if let Some(h) = &handle {
                        let log_data = serde_json::json!({
                            "sessionId": conn_id,
                            "connectionId": conn_id,
                            "type": "stderr",
                            "message": line
                        });
                        let _ = h.emit_all("session-log", &log_data);
                    }
                }
            });
        }

        // Spawn task to monitor process exit
        let conn_id_exit = connection_id.clone();
        let handle_exit = app_handle.clone();
        tokio::spawn(async move {
            match child.wait().await {
                Ok(status) => {
                    println!("[PROCESS] App exited with status: {:?}", status);
                    if let Some(h) = &handle_exit {
                        let _ = h.emit_all("session-removed", &conn_id_exit);
                    }
                }
                Err(e) => eprintln!("[PROCESS] Error waiting for app: {}", e),
            }
        });

        Ok(DebugConnection {
            connection_id,
            app_id: app.id.clone(),
            target_id: self.id.clone(),
            debug_ports: DebugPorts {
                node: Some(node_port),
                renderer: Some(renderer_port),
                websocket: None,
            },
            process_handle: pid,
            connection: crate::targets::types::ConnectionInfo {
                conn_type: "local-process".to_string(),
                node_port: Some(node_port),
                window_port: Some(renderer_port),
                websocket_url: None,
                debug_urls: None,
            },
        })
    }

    async fn disconnect(&self, _connection_id: &str) -> Result<()> {
        // Cleanup will be handled when the process terminates
        Ok(())
    }

    async fn is_available(&self) -> bool {
        true
    }
}
