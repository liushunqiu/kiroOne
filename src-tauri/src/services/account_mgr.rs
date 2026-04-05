use crate::models::account::Account;
use crate::db::queries;
use rusqlite::Connection;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AccountManager {
    db_conn: Arc<Mutex<Connection>>,
}

impl AccountManager {
    pub fn new(db_conn: Connection) -> Self {
        Self { 
            db_conn: Arc::new(Mutex::new(db_conn)),
        }
    }

    pub async fn import_account(&self, account: Account) -> Result<()> {
        let conn = self.db_conn.lock().await;
        queries::insert_account(&conn, &account)?;
        Ok(())
    }

    pub async fn get_all_accounts(&self) -> Result<Vec<Account>> {
        let conn = self.db_conn.lock().await;
        let accounts = queries::get_all_accounts(&conn)?;
        Ok(accounts)
    }

    pub async fn get_active_account(&self) -> Result<Option<Account>> {
        let conn = self.db_conn.lock().await;
        let account = queries::get_active_account(&conn)?;
        Ok(account)
    }

    pub async fn switch_account(&self, account_id: &str) -> Result<()> {
        let conn = self.db_conn.lock().await;
        queries::set_active_account(&conn, account_id)?;
        Ok(())
    }

    pub async fn delete_account(&self, account_id: &str) -> Result<()> {
        let conn = self.db_conn.lock().await;
        queries::delete_account(&conn, account_id)?;
        Ok(())
    }
}
