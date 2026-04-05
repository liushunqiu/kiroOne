use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub fn init_db(db_path: &PathBuf) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    
    // 创建账号表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS accounts (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            account_type TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            access_token TEXT,
            token_expires_at TEXT,
            main_quota REAL DEFAULT 0.0,
            main_quota_max REAL DEFAULT 0.0,
            trial_quota REAL DEFAULT 0.0,
            trial_quota_max REAL DEFAULT 0.0,
            bonus_quota REAL DEFAULT 0.0,
            bonus_quota_max REAL DEFAULT 0.0,
            tags TEXT DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            is_active INTEGER DEFAULT 0
        )",
        [],
    )?;

    // 创建配置表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS configs (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    Ok(conn)
}
