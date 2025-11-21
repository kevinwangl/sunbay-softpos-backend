use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 交易类型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum TransactionType {
    #[serde(rename = "PAYMENT")]
    Payment,
    #[serde(rename = "REFUND")]
    Refund,
    #[serde(rename = "VOID")]
    Void,
    #[serde(rename = "PREAUTH")]
    PreAuth,
    #[serde(rename = "CAPTURE")]
    Capture,
}

/// 交易状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum TransactionStatus {
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "APPROVED")]
    Approved,
    #[serde(rename = "DECLINED")]
    Declined,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "VOIDED")]
    Voided,
}

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: String,
    pub device_id: String,
    pub transaction_type: TransactionType,
    pub amount: i64, // 金额（分）
    pub currency: String,
    pub status: TransactionStatus,
    pub encrypted_pin_block: Option<String>,
    pub ksn: String,
    pub card_number_masked: Option<String>,
    pub merchant_id: Option<String>,
    pub terminal_id: Option<String>,
    pub authorization_code: Option<String>,
    pub response_code: Option<String>,
    pub response_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Transaction {
    /// 创建新的交易记录
    pub fn new(
        device_id: String,
        transaction_type: TransactionType,
        amount: i64,
        currency: String,
        ksn: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            device_id,
            transaction_type,
            amount,
            currency,
            status: TransactionStatus::Pending,
            encrypted_pin_block: None,
            ksn,
            card_number_masked: None,
            merchant_id: None,
            terminal_id: None,
            authorization_code: None,
            response_code: None,
            response_message: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
