use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeSettings {
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// 获取 Claude Code settings.json 路径
fn get_claude_settings_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "无法获取用户主目录".to_string())?;

    Ok(PathBuf::from(home).join(".claude").join("settings.json"))
}

/// 读取 Claude Code 配置
pub fn read_claude_settings() -> Result<ClaudeSettings, String> {
    let path = get_claude_settings_path()?;

    if !path.exists() {
        return Err(format!("Claude Code 配置文件不存在: {:?}", path));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))
}

/// 写入 Claude Code 配置
pub fn write_claude_settings(settings: &ClaudeSettings) -> Result<(), String> {
    let path = get_claude_settings_path()?;

    // 确保目录存在
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    std::fs::write(&path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

/// 同步 Kiro One 网关配置到 Claude Code
pub fn sync_to_claude(port: u16, api_key: &str) -> Result<(), String> {
    let mut settings = read_claude_settings()
        .unwrap_or_else(|_| ClaudeSettings {
            env: HashMap::new(),
        });

    // 更新环境变量
    settings.env.insert(
        "ANTHROPIC_BASE_URL".to_string(),
        format!("http://127.0.0.1:{}", port),
    );
    settings.env.insert(
        "ANTHROPIC_API_KEY".to_string(),
        api_key.to_string(),
    );

    write_claude_settings(&settings)?;
    Ok(())
}

/// 从 Claude Code 读取配置
pub fn read_from_claude() -> Result<(String, String), String> {
    let settings = read_claude_settings()?;

    let base_url = settings.env.get("ANTHROPIC_BASE_URL")
        .ok_or("配置中缺少 ANTHROPIC_BASE_URL")?;
    let api_key = settings.env.get("ANTHROPIC_API_KEY")
        .ok_or("配置中缺少 ANTHROPIC_API_KEY")?;

    Ok((base_url.clone(), api_key.clone()))
}
