use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::AppHandle;

pub struct DataStore {
    data_dir: PathBuf,
}

impl DataStore {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("获取应用数据目录失败: {}", e))?;

        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("创建数据目录失败: {}", e))?;

        Ok(Self { data_dir })
    }

    pub fn load<T: for<'de> Deserialize<'de>>(&self, filename: &str) -> Result<T, String> {
        let path = self.data_dir.join(filename);
        if !path.exists() {
            return Err(format!("文件不存在: {}", filename));
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取文件 {} 失败: {}", filename, e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("解析文件 {} 失败: {}", filename, e))
    }

    pub fn save<T: Serialize>(&self, filename: &str, data: &T) -> Result<(), String> {
        let path = self.data_dir.join(filename);

        // 使用临时文件实现原子写入
        let temp_path = self.data_dir.join(format!("{}.tmp", filename));

        let content = serde_json::to_string_pretty(data)
            .map_err(|e| format!("序列化数据失败: {}", e))?;

        std::fs::write(&temp_path, content)
            .map_err(|e| format!("写入临时文件失败: {}", e))?;

        std::fs::rename(&temp_path, &path)
            .map_err(|e| format!("重命名文件失败: {}", e))?;

        Ok(())
    }

    pub fn get_data_dir(&self) -> &PathBuf {
        &self.data_dir
    }
}
