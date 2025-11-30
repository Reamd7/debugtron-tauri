use crate::targets::registry::TargetRegistry;
use crate::targets::types::{AppInfo, DebugConnection, RemoteDeviceOptions};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct SessionState {
    pub connection: DebugConnection,
    pub active: bool, // Whether we've successfully discovered debug targets
    pub poll_count: u32, // Number of polling attempts
}

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<Mutex<TargetRegistry>>,
    pub sessions: Arc<Mutex<HashMap<String, SessionState>>>,
    pub apps: Arc<Mutex<HashMap<String, AppInfo>>>,
    pub app_handle: tauri::AppHandle,
    pub poll_trigger: Arc<tokio::sync::Notify>, // For immediate polling trigger
}

impl AppState {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            registry: Arc::new(Mutex::new(TargetRegistry::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            apps: Arc::new(Mutex::new(HashMap::new())),
            app_handle,
            poll_trigger: Arc::new(tokio::sync::Notify::new()),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        println!("[STATE] Initializing app state...");

        // Initialize local targets
        let mut registry = self.registry.lock().await;
        println!("[STATE] Initializing local targets...");
        registry.initialize_local_targets().await?;
        drop(registry);

        // Get all registered targets and emit events
        let registry = self.registry.lock().await;
        let targets = registry.get_all();
        println!("[STATE] Found {} targets", targets.len());

        for target in &targets {
            // Emit target-registered event
            let target_info = crate::targets::types::TargetInfo {
                id: target.get_id(),
                r#type: match target.get_type() {
                    crate::targets::types::TargetType::Local => "local",
                    crate::targets::types::TargetType::Remote => "remote",
                }.to_string(),
                name: target.get_name(),
                status: "connected".to_string(),
                last_discovery: Some(chrono::Utc::now().timestamp_millis() as u64),
            };

            println!("[STATE] Emitting target-registered event for: {}", target_info.name);
            match self.app_handle.emit_all("target-registered", &target_info) {
                Ok(_) => println!("[STATE] Successfully emitted target-registered event"),
                Err(e) => eprintln!("[STATE] Failed to emit target-registered event: {}", e),
            }
        }

        // Discover apps from all targets
        for target in targets {
            println!("[STATE] Discovering apps from target: {}", target.get_id());
            match target.discover_apps().await {
                Ok(apps) => {
                    println!("[STATE] Found {} apps", apps.len());

                    // Update global state
                    let mut apps_state = self.apps.lock().await;
                    for app in &apps {
                        println!("[STATE] Adding app: {}", app.name);
                        apps_state.insert(app.id.clone(), app.clone());
                    }
                    drop(apps_state);

                    // Emit to frontend
                    println!("[STATE] Emitting apps-updated event with {} apps", apps.len());
                    match self.app_handle.emit_all("apps-updated", &apps) {
                        Ok(_) => println!("[STATE] Successfully emitted apps-updated event"),
                        Err(e) => eprintln!("[STATE] Failed to emit apps-updated event: {}", e),
                    }
                }
                Err(e) => {
                    eprintln!("[STATE] Failed to discover apps: {}", e);
                }
            }
        }

        // Start polling task
        println!("[STATE] Starting polling task...");
        self.start_polling();

        println!("[STATE] Initialization complete");
        Ok(())
    }

    pub async fn debug_app(&self, app_info: &AppInfo, inspect_brk: Option<bool>) -> Result<()> {
        let registry = self.registry.lock().await;
        let target = registry
            .get_by_id(&app_info.target_id)
            .ok_or_else(|| anyhow::anyhow!("Target not found"))?;

        // Launch the app with app_handle for log forwarding
        let mut options = crate::targets::types::LaunchOptions::default();
        options.app_handle = Some(self.app_handle.clone());
        options.inspect_brk = inspect_brk;

        let connection = target
            .launch(app_info, options)
            .await?;

        let session_id = connection.connection_id.clone();

        // Store session with initial state
        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id.clone(), SessionState {
            connection: connection.clone(),
            active: false,
            poll_count: 0,
        });
        println!("[STATE] Session added with ID: {}", session_id);
        println!("[STATE] Current sessions: {:?}", sessions.keys().collect::<Vec<_>>());
        drop(sessions);

        // Emit to frontend
        println!("[STATE] Emitting session-added event with connectionId: {}", session_id);
        let _ = self
            .app_handle
            .emit_all("session-added", &connection);

        // Trigger immediate polling for the new session
        println!("[STATE] Triggering immediate polling for new session");
        self.poll_trigger.notify_one();

        Ok(())
    }

    pub async fn add_remote_device(&self, _options: RemoteDeviceOptions) -> Result<()> {
        // TODO: Implement remote device support
        Err(anyhow::anyhow!("Remote devices not yet implemented"))
    }

    pub async fn remove_device(&self, target_id: &str) -> Result<()> {
        let mut registry = self.registry.lock().await;
        registry.unregister(target_id);

        // Remove apps from this device
        let mut apps = self.apps.lock().await;
        apps.retain(|_, app| app.target_id != target_id);
        drop(apps);

        // Emit to frontend
        let _ = self.app_handle.emit_all("device-removed", target_id);

        Ok(())
    }

    pub async fn refresh_apps(&self, target_id: &str) -> Result<()> {
        let registry = self.registry.lock().await;
        let target = registry
            .get_by_id(target_id)
            .ok_or_else(|| anyhow::anyhow!("Target not found"))?;

        let apps = target.discover_apps().await?;

        // Update global state
        let mut apps_state = self.apps.lock().await;
        // Remove old apps from this device
        apps_state.retain(|_, app| app.target_id != target_id);
        // Add new apps
        for app in &apps {
            apps_state.insert(app.id.clone(), app.clone());
        }
        drop(apps_state);

        // Emit to frontend
        let _ = self.app_handle.emit_all("apps-updated", &apps);

        Ok(())
    }

    fn start_polling(&self) {
        let state = self.clone();
        tauri::async_runtime::spawn(async move {
            use tokio::time::{sleep, Duration};

            loop {
                // Wait for either timeout or immediate trigger
                tokio::select! {
                    _ = sleep(Duration::from_millis(100)) => {
                        // Regular polling cycle
                    }
                    _ = state.poll_trigger.notified() => {
                        println!("[POLLING] Immediate poll triggered");
                    }
                }

                // Poll debug endpoints
                let mut sessions = state.sessions.lock().await;
                let mut all_pages: std::collections::HashMap<String, Vec<serde_json::Value>> = std::collections::HashMap::new();

                for (session_id, session_state) in sessions.iter_mut() {
                    let connection = &session_state.connection;
                    session_state.poll_count += 1;

                    // Calculate adaptive polling interval based on state
                    let should_poll = if !session_state.active {
                        // Aggressive polling for new sessions: always poll
                        true
                    } else {
                        // Once active, poll every 30 cycles (3 seconds at 100ms interval)
                        session_state.poll_count % 30 == 0
                    };

                    if !should_poll {
                        continue;
                    }

                    let mut session_pages: Vec<serde_json::Value> = Vec::new();
                    let mut found_any = false;

                    // Poll Node.js inspector port
                    if let Some(node_port) = connection.debug_ports.node {
                        match reqwest::get(format!("http://127.0.0.1:{}/json", node_port)).await {
                            Ok(response) => {
                                if let Ok(text) = response.text().await {
                                    if let Ok(pages) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                                        if !pages.is_empty() {
                                            println!("[POLLING] Node inspector found {} targets for session {}", pages.len(), session_id);
                                            session_pages.extend(pages);
                                            found_any = true;
                                        }
                                    }
                                }
                            }
                            Err(_e) => {
                                if !session_state.active {
                                    println!("[POLLING] Waiting for node port {} to be ready...", node_port);
                                }
                            }
                        }
                    }

                    // Poll Chrome DevTools port (renderer)
                    if let Some(renderer_port) = connection.debug_ports.renderer {
                        match reqwest::get(format!("http://127.0.0.1:{}/json", renderer_port)).await {
                            Ok(response) => {
                                if let Ok(text) = response.text().await {
                                    if let Ok(pages) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                                        if !pages.is_empty() {
                                            println!("[POLLING] Renderer inspector found {} targets for session {}", pages.len(), session_id);
                                            session_pages.extend(pages);
                                            found_any = true;
                                        }
                                    }
                                }
                            }
                            Err(_e) => {
                                if !session_state.active {
                                    println!("[POLLING] Waiting for renderer port {} to be ready...", renderer_port);
                                }
                            }
                        }
                    }

                    // Mark session as active if we found any debug targets
                    if found_any && !session_state.active {
                        println!("[POLLING] Session {} is now active!", session_id);
                        session_state.active = true;
                    }

                    if !session_pages.is_empty() {
                        all_pages.insert(session_id.clone(), session_pages);
                    }
                }
                drop(sessions);

                // Emit combined pages if we have any
                if !all_pages.is_empty() {
                    match serde_json::to_string(&all_pages) {
                        Ok(json_str) => {
                            println!("[POLLING] Emitting pages-updated with {} sessions", all_pages.len());
                            println!("[POLLING] Session IDs in payload: {:?}", all_pages.keys().collect::<Vec<_>>());
                            let _ = state.app_handle.emit_all("pages-updated", &json_str);
                        }
                        Err(e) => {
                            eprintln!("[POLLING] Failed to serialize pages: {}", e);
                        }
                    }
                }
            }
        });
    }
}
