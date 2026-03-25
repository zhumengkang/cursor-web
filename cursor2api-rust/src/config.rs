use anyhow::{Context, Result};
use std::fs;
use crate::types::HotConfig;

pub fn read_config(config_path: &str) -> Result<HotConfig> {
    if !std::path::Path::new(config_path).exists() {
        return Ok(HotConfig::default());
    }
    let content = fs::read_to_string(config_path)
        .with_context(|| format!("读取配置文件失败: {}", config_path))?;
    let config: HotConfig = serde_yaml::from_str(&content)
        .with_context(|| "解析 config.yaml 失败")?;
    Ok(config)
}

pub fn write_config(config_path: &str, config: &HotConfig) -> Result<()> {
    let content = serde_yaml::to_string(config)
        .context("序列化配置失败")?;
    // 确保父目录存在
    if let Some(parent) = std::path::Path::new(config_path).parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(config_path, content)
        .with_context(|| format!("写入配置文件失败: {}", config_path))?;
    Ok(())
}
