use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 威胁事件
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ThreatEvent {
    pub id: String,
    pub device_id: String,
    pub threat_type: ThreatType,
    pub severity: ThreatSeverity,
    pub status: ThreatStatus,
    pub description: String,
    pub detected_at: String,
    pub resolved_at: Option<String>,
    pub resolved_by: Option<String>,
}

impl ThreatEvent {
    /// 创建新的威胁事件
    pub fn new(
        device_id: String,
        threat_type: ThreatType,
        severity: ThreatSeverity,
        description: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            device_id,
            threat_type,
            severity,
            status: ThreatStatus::Active,
            description,
            detected_at: chrono::Utc::now().to_rfc3339(),
            resolved_at: None,
            resolved_by: None,
        }
    }

    /// 解决威胁
    pub fn resolve(mut self, resolved_by: String) -> Self {
        self.status = ThreatStatus::Resolved;
        self.resolved_at = Some(chrono::Utc::now().to_rfc3339());
        self.resolved_by = Some(resolved_by);
        self
    }
}

/// 威胁类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "PascalCase")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThreatType {
    RootDetection,
    BootloaderUnlock,
    SystemTamper,
    AppTamper,
    TeeCompromise,
    LowSecurityScore,
    ConsecutiveLowScores,
    Other,
}

impl std::fmt::Display for ThreatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreatType::RootDetection => write!(f, "RootDetection"),
            ThreatType::BootloaderUnlock => write!(f, "BootloaderUnlock"),
            ThreatType::SystemTamper => write!(f, "SystemTamper"),
            ThreatType::AppTamper => write!(f, "AppTamper"),
            ThreatType::TeeCompromise => write!(f, "TeeCompromise"),
            ThreatType::LowSecurityScore => write!(f, "LowSecurityScore"),
            ThreatType::ConsecutiveLowScores => write!(f, "ConsecutiveLowScores"),
            ThreatType::Other => write!(f, "Other"),
        }
    }
}

/// 威胁严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "PascalCase")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for ThreatSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreatSeverity::Low => write!(f, "Low"),
            ThreatSeverity::Medium => write!(f, "Medium"),
            ThreatSeverity::High => write!(f, "High"),
            ThreatSeverity::Critical => write!(f, "Critical"),
        }
    }
}

/// 威胁状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "PascalCase")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThreatStatus {
    Active,
    Resolved,
}

impl std::fmt::Display for ThreatStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreatStatus::Active => write!(f, "Active"),
            ThreatStatus::Resolved => write!(f, "Resolved"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_threat_event() {
        let threat = ThreatEvent::new(
            "device123".to_string(),
            ThreatType::RootDetection,
            ThreatSeverity::High,
            "Root detected on device".to_string(),
        );

        assert_eq!(threat.device_id, "device123");
        assert_eq!(threat.threat_type, ThreatType::RootDetection);
        assert_eq!(threat.severity, ThreatSeverity::High);
        assert_eq!(threat.status, ThreatStatus::Active);
        assert!(threat.resolved_at.is_none());
    }

    #[test]
    fn test_resolve_threat() {
        let threat = ThreatEvent::new(
            "device123".to_string(),
            ThreatType::RootDetection,
            ThreatSeverity::High,
            "Root detected".to_string(),
        );

        let resolved = threat.resolve("admin".to_string());

        assert_eq!(resolved.status, ThreatStatus::Resolved);
        assert!(resolved.resolved_at.is_some());
        assert_eq!(resolved.resolved_by, Some("admin".to_string()));
    }
}
