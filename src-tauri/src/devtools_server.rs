use anyhow::Result;
use axum::Router;
use std::fs;
use std::io;
use std::path::PathBuf;
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, services::ServeDir};

/// 获取 DevTools 前端临时目录
fn get_devtools_temp_dir() -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join("debugtron-devtools");
    Ok(temp_dir)
}

/// 解压 DevTools 前端资源
fn extract_devtools(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    let temp_dir = get_devtools_temp_dir()?;

    // 如果目录已存在，先清理
    if temp_dir.exists() {
        println!("[DEVTOOLS] Cleaning existing temp directory: {:?}", temp_dir);
        fs::remove_dir_all(&temp_dir)?;
    }

    // 创建临时目录
    fs::create_dir_all(&temp_dir)?;

    // 获取打包的 zip 资源路径
    let resource_path = app_handle
        .path_resolver()
        .resolve_resource("resources/devtools-frontend.zip")
        .ok_or_else(|| anyhow::anyhow!("DevTools frontend resource not found"))?;

    println!("[DEVTOOLS] Extracting from: {:?}", resource_path);
    println!("[DEVTOOLS] Extracting to: {:?}", temp_dir);

    // 解压 zip 文件
    let file = fs::File::open(&resource_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = temp_dir.join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    println!("[DEVTOOLS] Extraction complete");
    Ok(temp_dir)
}

/// 清理 DevTools 临时目录
pub fn cleanup_devtools_temp() -> Result<()> {
    let temp_dir = get_devtools_temp_dir()?;
    if temp_dir.exists() {
        println!("[DEVTOOLS] Cleaning up temp directory: {:?}", temp_dir);
        fs::remove_dir_all(&temp_dir)?;
    }
    Ok(())
}

/// DevTools Server 管理器
pub struct DevToolsServer {
    port: u16,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    devtools_path: PathBuf,
}

impl DevToolsServer {
    /// 创建新的 DevTools 服务器实例
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        // 解压 DevTools 前端到临时目录
        let devtools_path = extract_devtools(app_handle)?;

        Ok(Self {
            port: 0,
            shutdown_tx: None,
            devtools_path,
        })
    }

    /// 启动服务器（提供本地构建的 DevTools 前端）
    pub async fn start(&mut self) -> Result<u16> {
        // 分配端口
        let port = portpicker::pick_unused_port().ok_or_else(|| {
            anyhow::anyhow!("Failed to find an unused port for DevTools server")
        })?;

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(tx);
        self.port = port;

        println!("[DEVTOOLS] Serving DevTools from: {:?}", self.devtools_path);

        // 验证路径是否存在
        if !self.devtools_path.exists() {
            return Err(anyhow::anyhow!("DevTools path does not exist: {:?}", self.devtools_path));
        }

        let devtools_path = self.devtools_path.clone();

        // 创建路由 - 提供静态文件服务
        let app = Router::new()
            .nest_service("/", ServeDir::new(devtools_path))
            .layer(CorsLayer::permissive());

        let addr = format!("127.0.0.1:{}", port);
        println!("[DEVTOOLS] Starting DevTools frontend server on {}", addr);

        // 启动服务器
        tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(&addr)
                .await
                .expect("Failed to bind DevTools server");

            println!("[DEVTOOLS] Frontend server listening on {}", addr);

            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    rx.await.ok();
                    println!("[DEVTOOLS] Server shutting down");
                })
                .await
                .expect("DevTools server failed");
        });

        Ok(port)
    }

    /// 停止服务器
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }

    /// 获取服务器端口
    pub fn port(&self) -> u16 {
        self.port
    }
}


/// 全局 DevTools 服务器实例
static DEVTOOLS_SERVER: Mutex<Option<DevToolsServer>> = Mutex::const_new(None);

/// 获取或启动 DevTools 服务器
pub async fn get_or_start_server(app_handle: &tauri::AppHandle) -> Result<u16> {
    let mut server_guard = DEVTOOLS_SERVER.lock().await;

    if let Some(server) = server_guard.as_ref() {
        // 服务器已启动
        Ok(server.port())
    } else {
        // 启动新服务器
        let mut server = DevToolsServer::new(app_handle)?;
        let port = server.start().await?;
        *server_guard = Some(server);
        Ok(port)
    }
}
