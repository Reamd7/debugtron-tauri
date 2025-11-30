use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetType {
    Local,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub icon: String,
    #[serde(rename = "exePath")]
    pub exe_path: Option<String>,
    #[serde(rename = "targetId")]
    pub target_id: String,
    #[serde(rename = "targetType")]
    pub target_type: TargetType,
    pub metadata: Option<AppMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    pub platform: Option<String>,
    #[serde(rename = "deviceInfo")]
    pub device_info: Option<String>,
    #[serde(rename = "packageName")]
    pub package_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LaunchOptions {
    #[serde(rename = "debugFlags")]
    pub debug_flags: Option<Vec<String>>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub cwd: Option<String>,
    #[serde(rename = "inspectBrk")]
    pub inspect_brk: Option<bool>,
    #[serde(skip)]
    pub app_handle: Option<tauri::AppHandle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConnection {
    #[serde(rename = "connectionId")]
    pub connection_id: String,
    #[serde(rename = "appId")]
    pub app_id: String,
    #[serde(rename = "targetId")]
    pub target_id: String,
    #[serde(rename = "debugPorts")]
    pub debug_ports: DebugPorts,
    #[serde(rename = "processHandle")]
    pub process_handle: Option<u32>,
    pub connection: ConnectionInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    #[serde(rename = "type")]
    pub conn_type: String,
    #[serde(rename = "nodePort", skip_serializing_if = "Option::is_none")]
    pub node_port: Option<u16>,
    #[serde(rename = "windowPort", skip_serializing_if = "Option::is_none")]
    pub window_port: Option<u16>,
    #[serde(rename = "websocketUrl", skip_serializing_if = "Option::is_none")]
    pub websocket_url: Option<String>,
    #[serde(rename = "debugUrls", skip_serializing_if = "Option::is_none")]
    pub debug_urls: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugPorts {
    pub node: Option<u16>,
    pub renderer: Option<u16>,
    pub websocket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteDeviceOptions {
    #[serde(rename = "type")]
    pub device_type: String,
    pub address: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub name: String,
    pub status: String,
    #[serde(rename = "lastDiscovery")]
    pub last_discovery: Option<u64>,
}

#[async_trait]
pub trait TargetAdapter: Send + Sync {
    fn get_type(&self) -> TargetType;
    fn get_id(&self) -> String;
    fn get_name(&self) -> String;

    async fn discover_apps(&self) -> Result<Vec<AppInfo>>;
    async fn launch(&self, app: &AppInfo, options: LaunchOptions) -> Result<DebugConnection>;
    async fn disconnect(&self, connection_id: &str) -> Result<()>;
    async fn is_available(&self) -> bool;
}
