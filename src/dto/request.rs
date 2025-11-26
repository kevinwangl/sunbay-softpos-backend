use serde::{Deserialize, Serialize};
use crate::models::{DeviceMode, TeeType, TransactionType, UpdateType};

/// 设备注册请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDeviceRequest {
    pub imei: String,
    pub model: String,
    pub os_version: String,
    pub tee_type: TeeType,
    pub public_key: String,
    #[serde(default = "default_device_mode")]
    pub device_mode: DeviceMode,
}

fn default_device_mode() -> DeviceMode {
    DeviceMode::FullPos
}

impl RegisterDeviceRequest {
    /// 验证请求数据
    pub fn validate(&self) -> Result<(), String> {
        // IMEI长度验证（15位）
        if self.imei.len() != 15 {
            return Err("IMEI must be 15 digits".to_string());
        }

        // IMEI必须是数字
        if !self.imei.chars().all(|c| c.is_ascii_digit()) {
            return Err("IMEI must contain only digits".to_string());
        }

        // 型号不能为空
        if self.model.trim().is_empty() {
            return Err("Model cannot be empty".to_string());
        }

        // 公钥不能为空
        if self.public_key.trim().is_empty() {
            return Err("Public key cannot be empty".to_string());
        }

        Ok(())
    }
}

/// 用户登录请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

impl LoginRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.username.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }

        if self.password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }

        Ok(())
    }
}

/// 健康检查提交请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckRequest {
    pub device_id: String,
    pub root_detection: bool,
    pub emulator_detection: bool,
    pub debugger_detection: bool,
    pub hook_detection: bool,
    pub tampering_detection: bool,
    pub signature: String,
}

impl HealthCheckRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.device_id.trim().is_empty() {
            return Err("Device ID cannot be empty".to_string());
        }

        if self.signature.trim().is_empty() {
            return Err("Signature cannot be empty".to_string());
        }

        Ok(())
    }
}

/// 密钥注入请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectKeyRequest {
    pub device_id: String,
}

impl InjectKeyRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.device_id.trim().is_empty() {
            return Err("Device ID cannot be empty".to_string());
        }

        Ok(())
    }
}

/// 密钥更新请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateKeyRequest {
    pub device_id: String,
}

impl UpdateKeyRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.device_id.trim().is_empty() {
            return Err("Device ID cannot be empty".to_string());
        }

        Ok(())
    }
}

/// 设备审批请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveDeviceRequest {
    pub device_id: String,
    pub operator: String,
}

impl ApproveDeviceRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.device_id.trim().is_empty() {
            return Err("Device ID cannot be empty".to_string());
        }

        if self.operator.trim().is_empty() {
            return Err("Operator cannot be empty".to_string());
        }

        Ok(())
    }
}

/// 设备拒绝请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectDeviceRequest {
    pub device_id: String,
    pub operator: String,
    pub reason: String,
}

impl RejectDeviceRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.device_id.trim().is_empty() {
            return Err("Device ID cannot be empty".to_string());
        }

        if self.operator.trim().is_empty() {
            return Err("Operator cannot be empty".to_string());
        }

        if self.reason.trim().is_empty() {
            return Err("Reason cannot be empty".to_string());
        }

        Ok(())
    }
}

/// 设备操作请求（暂停、吊销）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceOperationRequest {
    pub reason: String,
}

impl DeviceOperationRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.reason.trim().is_empty() {
            return Err("Reason cannot be empty".to_string());
        }

        if self.reason.trim().len() < 10 {
            return Err("Reason must be at least 10 characters".to_string());
        }

        Ok(())
    }
}

/// 交易鉴证请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestTransactionRequest {
    pub device_id: String,
    pub amount: i64,
    pub currency: String,
}

impl AttestTransactionRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.device_id.trim().is_empty() {
            return Err("Device ID cannot be empty".to_string());
        }

        if self.amount <= 0 {
            return Err("Amount must be positive".to_string());
        }

        if self.currency.trim().is_empty() {
            return Err("Currency cannot be empty".to_string());
        }

        Ok(())
    }
}

/// 交易处理请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTransactionRequest {
    pub device_id: String,
    pub transaction_type: TransactionType,
    pub amount: i64,
    pub currency: String,
    pub encrypted_pin_block: String,
    pub ksn: String,
    pub card_number_masked: Option<String>,
    pub transaction_token: String,
}

impl ProcessTransactionRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.device_id.trim().is_empty() {
            return Err("Device ID cannot be empty".to_string());
        }

        if self.amount <= 0 {
            return Err("Amount must be positive".to_string());
        }

        if self.encrypted_pin_block.trim().is_empty() {
            return Err("Encrypted PIN block cannot be empty".to_string());
        }

        if self.ksn.trim().is_empty() {
            return Err("KSN cannot be empty".to_string());
        }

        if self.transaction_token.trim().is_empty() {
            return Err("Transaction token cannot be empty".to_string());
        }

        Ok(())
    }
}

/// SDK版本创建请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVersionRequest {
    pub version: String,
    pub update_type: UpdateType,
    pub download_url: String,
    pub checksum: String,
    pub file_size: i64,
    pub release_notes: String,
    pub min_os_version: Option<String>,
}

impl CreateVersionRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty() {
            return Err("Version cannot be empty".to_string());
        }

        // 验证语义化版本号格式 (x.y.z)
        let parts: Vec<&str> = self.version.split('.').collect();
        if parts.len() != 3 {
            return Err("Version must be in format x.y.z".to_string());
        }

        for part in parts {
            if part.parse::<u32>().is_err() {
                return Err("Version parts must be numbers".to_string());
            }
        }

        if self.download_url.trim().is_empty() {
            return Err("Download URL cannot be empty".to_string());
        }

        if self.checksum.trim().is_empty() {
            return Err("Checksum cannot be empty".to_string());
        }

        if self.file_size <= 0 {
            return Err("File size must be positive".to_string());
        }

        Ok(())
    }
}

/// PINPad设备鉴证请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestPinpadRequest {
    pub device_id: String,
}

impl AttestPinpadRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.device_id.trim().is_empty() {
            return Err("Device ID cannot be empty".to_string());
        }

        Ok(())
    }
}

/// PIN加密请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptPinRequest {
    pub device_id: String,
    pub pin: String,
    pub attestation_token: String,
}

impl EncryptPinRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.device_id.trim().is_empty() {
            return Err("Device ID cannot be empty".to_string());
        }

        if self.pin.is_empty() {
            return Err("PIN cannot be empty".to_string());
        }

        // PIN长度验证（4-12位）
        if self.pin.len() < 4 || self.pin.len() > 12 {
            return Err("PIN must be 4-12 digits".to_string());
        }

        // PIN必须是数字
        if !self.pin.chars().all(|c| c.is_ascii_digit()) {
            return Err("PIN must contain only digits".to_string());
        }

        if self.attestation_token.trim().is_empty() {
            return Err("Attestation token cannot be empty".to_string());
        }

        Ok(())
    }
}

/// 版本更新请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVersionRequest {
    pub status: Option<crate::models::VersionStatus>,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
}

impl UpdateVersionRequest {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(url) = &self.download_url {
            if url.trim().is_empty() {
                return Err("Download URL cannot be empty".to_string());
            }
        }
        Ok(())
    }
}

/// 创建推送任务请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePushTaskRequest {
    pub version_id: String,
    pub target_devices: Option<Vec<String>>,
    pub filter_model: Option<String>,
}

