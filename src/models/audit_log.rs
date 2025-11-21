use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 操作结果
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum OperationResult {
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "FAILURE")]
    Failure,
    #[serde(rename = "PARTIAL")]
    Partial,
}

/// 审计日志
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: String,
    pub operation: String,
    pub operator: String,
    pub device_id: Option<String>,
    pub result: OperationResult,
    pub details: Option<String>, // JSON object
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: String,
}

impl AuditLog {
    /// 创建新的审计日志
    pub fn new(
        operation: String,
        operator: String,
        result: OperationResult,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            operation,
            operator,
            device_id: None,
            result,
            details: None,
            ip_address: None,
            user_agent: None,
            created_at: now,
        }
    }

    /// 设置设备ID
    pub fn with_device_id(mut self, device_id: String) -> Self {
        self.device_id = Some(device_id);
        self
    }

    /// 设置详细信息
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    /// 设置IP地址
    pub fn with_ip_address(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }

    /// 设置User Agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }
}
