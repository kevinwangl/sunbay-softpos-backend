use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 内核版本状态
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum KernelStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "stable")]
    Stable,
    #[serde(rename = "deprecated")]
    Deprecated,
}

impl std::fmt::Display for KernelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KernelStatus::Draft => write!(f, "draft"),
            KernelStatus::Stable => write!(f, "stable"),
            KernelStatus::Deprecated => write!(f, "deprecated"),
        }
    }
}

/// 内核版本模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Kernel {
    pub id: String,
    pub version: String,
    pub file_path: String,
    pub file_hash: String,
    pub file_size: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
