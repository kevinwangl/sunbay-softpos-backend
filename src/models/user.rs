use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 用户角色
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum UserRole {
    #[serde(rename = "ADMIN")]
    Admin,
    #[serde(rename = "OPERATOR")]
    Operator,
    #[serde(rename = "VIEWER")]
    Viewer,
}

/// 用户状态
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum UserStatus {
    #[serde(rename = "ACTIVE")]
    Active,
    #[serde(rename = "INACTIVE")]
    Inactive,
    #[serde(rename = "LOCKED")]
    Locked,
}

/// 用户
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub last_login_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl User {
    /// 创建新用户
    pub fn new(
        username: String,
        password_hash: String,
        email: String,
        role: UserRole,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            username,
            password_hash,
            email,
            role,
            status: UserStatus::Active,
            last_login_at: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
