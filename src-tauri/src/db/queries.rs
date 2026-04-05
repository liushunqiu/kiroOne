use rusqlite::{Connection, Result, params};
use crate::models::account::Account;
use chrono::Utc;

pub fn insert_account(conn: &Connection, account: &Account) -> Result<()> {
    conn.execute(
        "INSERT INTO accounts (
            id, name, email, account_type, refresh_token,
            access_token, token_expires_at, main_quota, main_quota_max,
            trial_quota, trial_quota_max, bonus_quota, bonus_quota_max,
            tags, created_at, updated_at, is_active
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        params![
            account.id,
            account.name,
            account.email,
            serde_json::to_string(&account.account_type).unwrap_or_default(),
            account.refresh_token,
            account.access_token,
            account.token_expires_at.map(|t| t.to_rfc3339()),
            account.quota.as_ref().map_or(0.0, |q| q.main_quota),
            account.quota.as_ref().map_or(0.0, |q| q.main_quota_max),
            account.quota.as_ref().map_or(0.0, |q| q.trial_quota),
            account.quota.as_ref().map_or(0.0, |q| q.trial_quota_max),
            account.quota.as_ref().map_or(0.0, |q| q.bonus_quota),
            account.quota.as_ref().map_or(0.0, |q| q.bonus_quota_max),
            serde_json::to_string(&account.tags).unwrap_or_default(),
            account.created_at.to_rfc3339(),
            account.updated_at.to_rfc3339(),
            if account.is_active { 1 } else { 0 }
        ],
    )?;
    Ok(())
}

pub fn get_all_accounts(conn: &Connection) -> Result<Vec<Account>> {
    let mut stmt = conn.prepare("SELECT * FROM accounts ORDER BY created_at DESC")?;
    let accounts = stmt.query_map([], |row| {
        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
            account_type: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
            refresh_token: row.get(4)?,
            access_token: row.get(5)?,
            token_expires_at: row.get::<_, Option<String>>(6)?.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|t| t.with_timezone(&Utc)),
            quota: Some(crate::models::account::Quota {
                main_quota: row.get(7)?,
                main_quota_max: row.get(8)?,
                trial_quota: row.get(9)?,
                trial_quota_max: row.get(10)?,
                bonus_quota: row.get(11)?,
                bonus_quota_max: row.get(12)?,
            }),
            tags: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or_default(),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(14)?).unwrap().with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(15)?).unwrap().with_timezone(&Utc),
            is_active: row.get::<_, i32>(16)? == 1,
        })
    })?;
    accounts.collect()
}

pub fn set_active_account(conn: &Connection, account_id: &str) -> Result<()> {
    // 先将所有账号设为非活跃
    conn.execute("UPDATE accounts SET is_active = 0", [])?;
    // 再设置指定账号为活跃
    conn.execute(
        "UPDATE accounts SET is_active = 1 WHERE id = ?1",
        [account_id],
    )?;
    Ok(())
}

pub fn get_active_account(conn: &Connection) -> Result<Option<Account>> {
    let mut stmt = conn.prepare("SELECT * FROM accounts WHERE is_active = 1 LIMIT 1")?;
    let mut accounts = stmt.query_map([], |row| {
        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
            account_type: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
            refresh_token: row.get(4)?,
            access_token: row.get(5)?,
            token_expires_at: row.get::<_, Option<String>>(6)?.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|t| t.with_timezone(&Utc)),
            quota: Some(crate::models::account::Quota {
                main_quota: row.get(7)?,
                main_quota_max: row.get(8)?,
                trial_quota: row.get(9)?,
                trial_quota_max: row.get(10)?,
                bonus_quota: row.get(11)?,
                bonus_quota_max: row.get(12)?,
            }),
            tags: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or_default(),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(14)?).unwrap().with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(15)?).unwrap().with_timezone(&Utc),
            is_active: row.get::<_, i32>(16)? == 1,
        })
    })?;
    
    if let Some(account) = accounts.next() {
        Ok(Some(account?))
    } else {
        Ok(None)
    }
}

pub fn delete_account(conn: &Connection, account_id: &str) -> Result<()> {
    conn.execute("DELETE FROM accounts WHERE id = ?1", [account_id])?;
    Ok(())
}
