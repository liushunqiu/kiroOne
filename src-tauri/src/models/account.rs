use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    pub account_type: AccountType,
    pub refresh_token: String,
    pub access_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub quota: Option<Quota>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountType {
    Social,
    IdC,
}

impl Default for AccountType {
    fn default() -> Self {
        AccountType::Social
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quota {
    pub main_quota: f64,
    pub main_quota_max: f64,
    pub trial_quota: f64,
    pub trial_quota_max: f64,
    pub bonus_quota: f64,
    pub bonus_quota_max: f64,
}

impl Account {
    pub fn is_token_expired(&self) -> bool {
        match self.token_expires_at {
            Some(expires_at) => Utc::now() >= expires_at,
            None => true,
        }
    }

    pub fn quota_percentage(&self) -> Option<f64> {
        self.quota.as_ref().map(|q| {
            let total = q.main_quota + q.trial_quota + q.bonus_quota;
            let total_max = q.main_quota_max + q.trial_quota_max + q.bonus_quota_max;
            if total_max == 0.0 {
                0.0
            } else {
                (total / total_max) * 100.0
            }
        })
    }
}
