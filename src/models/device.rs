use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 设备模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Device {
    pub id: String,
    pub imei: String,
    pub model: String,
    pub os_version: String,
    pub tee_type: String,
    pub device_mode: String,
    pub public_key: Vec<u8>,
    pub status: String,
    pub merchant_id: Option<String>,
    pub merchant_name: Option<String>,
    pub security_score: i32,
    pub current_ksn: String,
    pub ipek_injected_at: Option<String>,
    pub key_remaining_count: i32,
    pub key_total_count: i32,
    pub registered_at: String,
    pub approved_at: Option<String>,
    pub approved_by: Option<String>,
    pub last_active_at: Option<String>,
    pub updated_at: String,
    pub nfc_present: bool,
}

impl Device {
    pub fn new(
        imei: String,
        model: String,
        os_version: String,
        tee_type: crate::models::TeeType,
        public_key: Vec<u8>,
        device_mode: crate::models::DeviceMode,
        nfc_present: bool,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            imei,
            model,
            os_version,
            tee_type: tee_type.as_str().to_string(),
            device_mode: device_mode.as_str().to_string(),
            public_key,
            status: DeviceStatus::Pending.as_str().to_string(),
            merchant_id: None,
            merchant_name: None,
            security_score: 100,
            current_ksn: String::new(),
            ipek_injected_at: None,
            key_remaining_count: 0,
            key_total_count: 0,
            registered_at: now.clone(),
            approved_at: None,
            approved_by: None,
            last_active_at: None,
            updated_at: now,
            nfc_present,
        }
    }
}

/// TEE类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TeeType {
    Qtee,
    TrustZone,
}

impl TeeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TeeType::Qtee => "QTEE",
            TeeType::TrustZone => "TRUSTZONE",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "QTEE" => Some(TeeType::Qtee),
            "TRUSTZONE" => Some(TeeType::TrustZone),
            _ => None,
        }
    }
}

/// 设备模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceMode {
    /// 完整POS模式，支持完整交易处理
    FullPos,
    /// PINPad模式，仅作为PIN输入设备
    PinPad,
}

impl DeviceMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceMode::FullPos => "FULL_POS",
            DeviceMode::PinPad => "PINPAD",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "FULL_POS" => Some(DeviceMode::FullPos),
            "PINPAD" => Some(DeviceMode::PinPad),
            _ => None,
        }
    }
}

impl Default for DeviceMode {
    fn default() -> Self {
        DeviceMode::FullPos
    }
}

/// 设备状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceStatus {
    Pending,
    Active,
    Suspended,
    Revoked,
    Rejected,
}

impl DeviceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceStatus::Pending => "PENDING",
            DeviceStatus::Active => "ACTIVE",
            DeviceStatus::Suspended => "SUSPENDED",
            DeviceStatus::Revoked => "REVOKED",
            DeviceStatus::Rejected => "REJECTED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "PENDING" => Some(DeviceStatus::Pending),
            "ACTIVE" => Some(DeviceStatus::Active),
            "SUSPENDED" => Some(DeviceStatus::Suspended),
            "REVOKED" => Some(DeviceStatus::Revoked),
            "REJECTED" => Some(DeviceStatus::Rejected),
            _ => None,
        }
    }

    /// 检查状态转换是否有效
    pub fn can_transition_to(&self, new_status: DeviceStatus) -> bool {
        match (self, new_status) {
            // PENDING可以转换到ACTIVE或REJECTED
            (DeviceStatus::Pending, DeviceStatus::Active) => true,
            (DeviceStatus::Pending, DeviceStatus::Rejected) => true,

            // ACTIVE可以转换到SUSPENDED或REVOKED
            (DeviceStatus::Active, DeviceStatus::Suspended) => true,
            (DeviceStatus::Active, DeviceStatus::Revoked) => true,

            // SUSPENDED可以转换到ACTIVE或REVOKED
            (DeviceStatus::Suspended, DeviceStatus::Active) => true,
            (DeviceStatus::Suspended, DeviceStatus::Revoked) => true,

            // REVOKED和REJECTED是终态，不能转换
            (DeviceStatus::Revoked, _) => false,
            (DeviceStatus::Rejected, _) => false,

            // 其他转换无效
            _ => false,
        }
    }
}

impl Default for DeviceStatus {
    fn default() -> Self {
        DeviceStatus::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tee_type_conversion() {
        assert_eq!(TeeType::Qtee.as_str(), "QTEE");
        assert_eq!(TeeType::TrustZone.as_str(), "TRUSTZONE");

        assert_eq!(TeeType::from_str("QTEE"), Some(TeeType::Qtee));
        assert_eq!(TeeType::from_str("TRUSTZONE"), Some(TeeType::TrustZone));
        assert_eq!(TeeType::from_str("INVALID"), None);
    }

    #[test]
    fn test_device_mode_conversion() {
        assert_eq!(DeviceMode::FullPos.as_str(), "FULL_POS");
        assert_eq!(DeviceMode::PinPad.as_str(), "PINPAD");

        assert_eq!(DeviceMode::from_str("FULL_POS"), Some(DeviceMode::FullPos));
        assert_eq!(DeviceMode::from_str("PINPAD"), Some(DeviceMode::PinPad));
        assert_eq!(DeviceMode::from_str("INVALID"), None);
    }

    #[test]
    fn test_device_mode_default() {
        assert_eq!(DeviceMode::default(), DeviceMode::FullPos);
    }

    #[test]
    fn test_device_status_conversion() {
        assert_eq!(DeviceStatus::Pending.as_str(), "PENDING");
        assert_eq!(DeviceStatus::Active.as_str(), "ACTIVE");

        assert_eq!(DeviceStatus::from_str("PENDING"), Some(DeviceStatus::Pending));
        assert_eq!(DeviceStatus::from_str("ACTIVE"), Some(DeviceStatus::Active));
        assert_eq!(DeviceStatus::from_str("INVALID"), None);
    }

    #[test]
    fn test_device_status_transitions() {
        // Valid transitions from PENDING
        assert!(DeviceStatus::Pending.can_transition_to(DeviceStatus::Active));
        assert!(DeviceStatus::Pending.can_transition_to(DeviceStatus::Rejected));
        assert!(!DeviceStatus::Pending.can_transition_to(DeviceStatus::Suspended));

        // Valid transitions from ACTIVE
        assert!(DeviceStatus::Active.can_transition_to(DeviceStatus::Suspended));
        assert!(DeviceStatus::Active.can_transition_to(DeviceStatus::Revoked));
        assert!(!DeviceStatus::Active.can_transition_to(DeviceStatus::Pending));

        // Valid transitions from SUSPENDED
        assert!(DeviceStatus::Suspended.can_transition_to(DeviceStatus::Active));
        assert!(DeviceStatus::Suspended.can_transition_to(DeviceStatus::Revoked));

        // REVOKED is terminal
        assert!(!DeviceStatus::Revoked.can_transition_to(DeviceStatus::Active));
        assert!(!DeviceStatus::Revoked.can_transition_to(DeviceStatus::Pending));

        // REJECTED is terminal
        assert!(!DeviceStatus::Rejected.can_transition_to(DeviceStatus::Active));
        assert!(!DeviceStatus::Rejected.can_transition_to(DeviceStatus::Pending));
    }

    #[test]
    fn test_device_status_default() {
        assert_eq!(DeviceStatus::default(), DeviceStatus::Pending);
    }
}
