use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 更新类型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum UpdateType {
    #[serde(rename = "MANDATORY")]
    Mandatory,
    #[serde(rename = "OPTIONAL")]
    Optional,
    #[serde(rename = "SECURITY")]
    Security,
}

/// 版本状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum VersionStatus {
    #[serde(rename = "DRAFT")]
    Draft,
    #[serde(rename = "TESTING")]
    Testing,
    #[serde(rename = "RELEASED")]
    Released,
    #[serde(rename = "DEPRECATED")]
    Deprecated,
}

/// SDK版本
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SdkVersion {
    pub id: String,
    pub version: String,
    pub update_type: UpdateType,
    pub status: VersionStatus,
    pub download_url: String,
    pub checksum: String,
    pub file_size: i64,
    pub release_notes: String,
    pub min_os_version: Option<String>,
    pub target_devices: Option<String>, // JSON array of device IDs
    pub distribution_strategy: Option<String>, // JSON object
    pub created_at: String,
    pub released_at: Option<String>,
}

impl SdkVersion {
    /// 创建新的SDK版本
    pub fn new(
        version: String,
        update_type: UpdateType,
        download_url: String,
        checksum: String,
        file_size: i64,
        release_notes: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            version,
            update_type,
            status: VersionStatus::Draft,
            download_url,
            checksum,
            file_size,
            release_notes,
            min_os_version: None,
            target_devices: None,
            distribution_strategy: None,
            created_at: now,
            released_at: None,
        }
    }
}
