use std::fs;
use std::path::Path;
use std::process::{Child, Command};

/// 内嵌 node_bundle.tar.gz（由 build.rs 保证文件存在）
static NODE_BUNDLE: &[u8] = include_bytes!("../node_bundle.tar.gz");

/// bundle 版本号，每次更新 bundle 时修改此常量以触发重新解压
const BUNDLE_VERSION: &str = env!("CARGO_PKG_VERSION");

fn needs_extract(app_dir: &Path) -> bool {
    if !app_dir.exists() {
        return true;
    }
    // 如果 bundle 是空文件（本地开发占位），跳过解压
    if NODE_BUNDLE.is_empty() {
        return false;
    }
    let version_file = app_dir.join(".bundle_version");
    match fs::read_to_string(&version_file) {
        Ok(v) => v.trim() != BUNDLE_VERSION,
        Err(_) => true,
    }
}

fn extract_bundle(app_dir: &Path) {
    tracing::info!("解压 node_bundle 到 {:?}", app_dir);
    if app_dir.exists() {
        fs::remove_dir_all(app_dir).expect("清理旧 app 目录失败");
    }
    fs::create_dir_all(app_dir).expect("创建 app 目录失败");

    let cursor = std::io::Cursor::new(NODE_BUNDLE);
    let gz = flate2::read::GzDecoder::new(cursor);
    let mut archive = tar::Archive::new(gz);
    archive.unpack(app_dir).expect("解压 node_bundle.tar.gz 失败");

    // 写入版本文件
    fs::write(app_dir.join(".bundle_version"), BUNDLE_VERSION)
        .expect("写入 bundle 版本文件失败");

    // Unix 下设置 node 可执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let node_bin = app_dir.join("node");
        if node_bin.exists() {
            let mut perms = fs::metadata(&node_bin).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&node_bin, perms).unwrap();
        }
    }

    tracing::info!("node_bundle 解压完成");
}

/// 启动 cursor2api Node.js 服务
/// 返回子进程 Child，调用方负责保存以避免被 drop 后 kill
pub fn setup_and_spawn(exe_dir: &Path) -> Option<Child> {
    // 如果 bundle 是空占位文件（本地开发），不启动
    if NODE_BUNDLE.is_empty() {
        tracing::warn!("node_bundle.tar.gz 为空，跳过 cursor2api 启动（本地开发模式）");
        return None;
    }

    let app_dir = exe_dir.join("app");

    if needs_extract(&app_dir) {
        extract_bundle(&app_dir);
    } else {
        tracing::info!("node_bundle 已是最新版本，跳过解压");
    }

    let node_bin = if cfg!(windows) {
        app_dir.join("node.exe")
    } else {
        app_dir.join("node")
    };

    let dist_js = app_dir.join("dist").join("index.js");

    if !node_bin.exists() {
        tracing::error!("node 二进制不存在: {:?}", node_bin);
        return None;
    }
    if !dist_js.exists() {
        tracing::error!("dist/index.js 不存在: {:?}", dist_js);
        return None;
    }

    let config_path = exe_dir.join("config.yaml");
    let db_path = exe_dir.join("logs").join("cursor2api.db");
    let log_dir = exe_dir.join("logs");

    // 确保 logs 目录存在
    fs::create_dir_all(&log_dir).ok();

    tracing::info!("启动 cursor2api: {:?} {:?}", node_bin, dist_js);

    let child = Command::new(&node_bin)
        .arg(&dist_js)
        .env("PORT", "3010")
        .env("CONFIG_PATH", &config_path)
        .env("DB_PATH", &db_path)
        .env("LOG_DIR", &log_dir)
        .current_dir(exe_dir)
        .spawn()
        .expect("启动 cursor2api 失败");

    tracing::info!("cursor2api 已启动，PID={}", child.id());
    Some(child)
}
