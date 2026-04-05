use std::path::PathBuf;
use serde_json::json;
use anyhow::Result;

pub fn generate_claude_config(port: u16) -> Result<()> {
    let config = json!({
        "env": {
            "ANTHROPIC_BASE_URL": format!("http://127.0.0.1:{}", port),
            "ANTHROPIC_API_KEY": "sk-local-kiro-gateway"
        }
    });

    let config_path = get_claude_config_path()?;
    std::fs::write(
        &config_path,
        serde_json::to_string_pretty(&config)?
    )?;

    tracing::info!("Claude Code config written to: {}", config_path.display());
    Ok(())
}

fn get_claude_config_path() -> Result<PathBuf> {
    // 根据操作系统获取不同的配置路径
    let home_dir = home::home_dir().ok_or(anyhow::anyhow!("Cannot find home directory"))?;
    
    #[cfg(target_os = "windows")]
    let path = home_dir.join(".claude").join("settings.json");
    
    #[cfg(target_os = "macos")]
    let path = home_dir.join(".claude").join("settings.json");
    
    #[cfg(target_os = "linux")]
    let path = home_dir.join(".claude").join("settings.json");

    // 确保目录存在
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    Ok(path)
}

pub fn apply_claude_config(port: u16) -> Result<String> {
    generate_claude_config(port)?;
    Ok(format!("Claude Code 已配置为使用 http://127.0.0.1:{}", port))
}
