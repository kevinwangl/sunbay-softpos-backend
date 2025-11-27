use serde::{Deserialize, Serialize};
use crate::models::{
    Device, DeviceMode, DeviceStatus, TeeType, Transaction, TransactionStatus,
    SdkVersion, AuditLog, OperationResult,
};

/// 通用API响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// 设备注册响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDeviceResponse {
    pub device_id: String,
    pub ksn: String,
    pub status: DeviceStatus,
    pub message: String,
}

impl From<Device> for RegisterDeviceResponse {
    fn from(device: Device) -> Self {
        Self {
            device_id: device.id,
            ksn: device.current_ksn,
            status: DeviceStatus::from_str(&device.status).unwrap_or(DeviceStatus::Pending),
            message: "Device registered successfully. Awaiting approval.".to_string(),
        }
    }
}

/// 设备响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceResponse {
    pub id: String,
    pub imei: String,
    pub model: String,
    pub os_version: String,
    pub tee_type: TeeType,
    pub device_mode: DeviceMode,
    pub status: DeviceStatus,
    pub security_score: i32,
    pub ksn: Option<String>,
    pub key_injected_at: Option<String>,
    pub key_updated_at: Option<String>,
    pub key_usage_count: Option<i32>,
    pub key_max_usage: Option<i32>,
    pub registered_at: String,
    pub approved_at: Option<String>,
}

impl From<Device> for DeviceResponse {
    fn from(device: Device) -> Self {
        Self {
            id: device.id,
            imei: device.imei,
            model: device.model,
            os_version: device.os_version,
            tee_type: TeeType::from_str(&device.tee_type).unwrap_or(TeeType::Qtee),
            device_mode: DeviceMode::from_str(&device.device_mode).unwrap_or(DeviceMode::FullPos),
            status: DeviceStatus::from_str(&device.status).unwrap_or(DeviceStatus::Pending),
            security_score: device.security_score,
            ksn: Some(device.current_ksn),
            key_injected_at: device.ipek_injected_at,
            key_updated_at: Some(device.updated_at.clone()),
            key_usage_count: Some(device.key_total_count - device.key_remaining_count),
            key_max_usage: Some(device.key_total_count),
            registered_at: device.registered_at,
            approved_at: device.approved_at,
        }
    }
}

/// 设备列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceListResponse {
    pub devices: Vec<DeviceResponse>,
    pub total: i64,
}

/// 登录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    #[serde(rename = "token")]
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    #[serde(rename = "id")]
    pub user_id: String,
    pub username: String,
    pub role: String,
}

/// 健康检查响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub check_id: String,
    pub device_id: String,
    pub security_score: i32,
    pub recommended_action: String,
    pub threats_detected: Vec<String>,
    pub checked_at: String,
}

/// 健康检查概览响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthOverviewResponse {
    pub device_id: String,
    pub latest_score: i32,
    pub average_score: f64,
    pub total_checks: i64,
    pub last_check_at: Option<String>,
}

/// 密钥注入响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectKeyResponse {
    pub device_id: String,
    pub encrypted_ipek: String,
    pub ksn: String,
    pub injected_at: String,
    pub message: String,
}

/// 密钥状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyStatusResponse {
    pub device_id: String,
    #[serde(rename = "currentKSN")]
    pub current_ksn: String,
    pub remaining_count: i32,
    pub status: String,
    pub last_updated: String,
    pub next_update_required: Option<String>,
}

/// 密钥更新响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateKeyResponse {
    pub device_id: String,
    pub new_ksn: String,
    pub encrypted_ipek: String,
    pub updated_at: String,
    pub message: String,
}

/// 交易鉴证响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestTransactionResponse {
    pub transaction_token: String,
    pub device_id: String,
    pub expires_at: String,
    pub message: String,
}

/// 交易处理响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTransactionResponse {
    pub transaction_id: String,
    pub status: TransactionStatus,
    pub authorization_code: Option<String>,
    pub message: String,
}

/// 交易响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub id: String,
    pub device_id: String,
    pub transaction_type: String,
    pub amount: i64,
    pub currency: String,
    pub status: TransactionStatus,
    pub card_number_masked: Option<String>,
    pub authorization_code: Option<String>,
    pub created_at: String,
}

impl From<Transaction> for TransactionResponse {
    fn from(tx: Transaction) -> Self {
        Self {
            id: tx.id,
            device_id: tx.device_id,
            transaction_type: format!("{:?}", tx.transaction_type),
            amount: tx.amount,
            currency: tx.currency,
            status: tx.status,
            card_number_masked: tx.card_number_masked,
            authorization_code: tx.authorization_code,
            created_at: tx.created_at,
        }
    }
}

/// 交易列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionListResponse {
    pub transactions: Vec<TransactionResponse>,
    pub total: i64,
}

/// SDK版本响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResponse {
    pub id: String,
    pub version: String,
    pub update_type: String,
    pub status: String,
    pub download_url: String,
    pub file_size: i64,
    pub release_notes: String,
    pub created_at: String,
    pub released_at: Option<String>,
}

impl From<SdkVersion> for VersionResponse {
    fn from(version: SdkVersion) -> Self {
        Self {
            id: version.id,
            version: version.version,
            update_type: format!("{:?}", version.update_type),
            status: format!("{:?}", version.status),
            download_url: version.download_url,
            file_size: version.file_size,
            release_notes: version.release_notes,
            created_at: version.created_at,
            released_at: version.released_at,
        }
    }
}

/// 版本列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionListResponse {
    pub versions: Vec<VersionResponse>,
    pub total: i64,
}

/// 审计日志响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub id: String,
    pub operation: String,
    pub operator: String,
    pub device_id: Option<String>,
    pub result: OperationResult,
    pub details: Option<String>,
    pub created_at: String,
}

impl From<AuditLog> for AuditLogResponse {
    fn from(log: AuditLog) -> Self {
        Self {
            id: log.id,
            operation: log.operation,
            operator: log.operator,
            device_id: log.device_id,
            result: log.result,
            details: log.details,
            created_at: log.created_at,
        }
    }
}

/// 审计日志列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogListResponse {
    pub logs: Vec<AuditLogResponse>,
    pub total: i64,
}

/// 设备统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatisticsResponse {
    pub total: i64,
    pub active: i64,
    pub pending: i64,
    pub suspended: i64,
    pub revoked: i64,
    pub average_security_score: f64,
}

/// 威胁统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatStatisticsResponse {
    pub total: i64,
    pub active: i64,
    pub resolved: i64,
    pub critical: i64,
    pub high: i64,
    pub medium: i64,
    pub low: i64,
}

/// 系统健康检查响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthResponse {
    pub status: String,
    pub database: bool,
    pub redis: bool,
    pub timestamp: String,
}

/// PINPad鉴证响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestPinpadResponse {
    pub attestation_token: String,
    pub device_id: String,
    pub expires_at: String,
    pub message: String,
}

/// PIN加密响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptPinResponse {
    pub encrypted_pin_block: String,
    pub ksn: String,
    pub device_id: String,
    pub encrypted_at: String,
}

/// 健康检查列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckListResponse {
    pub checks: Vec<HealthCheckResponse>,
    pub total: i64,
}

/// 威胁响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatResponse {
    pub id: String,
    pub device_id: String,
    pub threat_type: String,
    pub severity: String,
    pub status: String,
    pub description: String,
    pub detected_at: String,
    pub resolved_at: Option<String>,
}

impl From<crate::models::ThreatEvent> for ThreatResponse {
    fn from(event: crate::models::ThreatEvent) -> Self {
        Self {
            id: event.id,
            device_id: event.device_id,
            threat_type: event.threat_type.to_string(),
            severity: event.severity.to_string(),
            status: event.status.to_string(),
            description: event.description,
            detected_at: event.detected_at,
            resolved_at: event.resolved_at,
        }
    }
}

/// 威胁列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatListResponse {
    pub threats: Vec<ThreatResponse>,
    pub total: i64,
}

/// 创建版本响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVersionResponse {
    pub id: String,
    pub version: String,
    pub status: crate::models::VersionStatus,
    pub message: String,
}

/// 仪表盘健康概览响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardHealthOverviewResponse {
    pub total_devices: i64,
    pub online_devices: i64,
    pub abnormal_devices: i64,
    pub average_security_score: f64,
    pub status_distribution: Vec<StatusDistribution>,
    pub score_distribution: Vec<ScoreDistribution>,
    pub recent_abnormal_devices: Vec<AbnormalDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDistribution {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDistribution {
    pub range: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbnormalDevice {
    pub id: String,
    pub merchant_name: String,
    pub security_score: i32,
    pub last_check_at: String,
}

