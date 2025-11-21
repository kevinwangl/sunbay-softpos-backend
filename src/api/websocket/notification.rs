use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use super::connection::{broadcast_message, ConnectionPool};

/// 通知类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    SecurityAlert,
    ThreatAlert,
    KeyWarning,
    DeviceStatusChange,
    SystemAlert,
}

/// 通知严重级别
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum NotificationSeverity {
    High,
    Medium,
    Low,
    Info,
}

/// 通知消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    #[serde(rename = "type")]
    pub notification_type: NotificationType,
    pub severity: NotificationSeverity,
    pub title: String,
    pub message: String,
    pub device_id: Option<String>,
    pub threat_id: Option<String>,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Notification {
    /// 创建安全告警通知
    pub fn security_alert(
        device_id: String,
        security_score: i32,
        message: String,
    ) -> Self {
        Self {
            notification_type: NotificationType::SecurityAlert,
            severity: if security_score < 60 {
                NotificationSeverity::High
            } else {
                NotificationSeverity::Medium
            },
            title: "设备安全告警".to_string(),
            message,
            device_id: Some(device_id.clone()),
            threat_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: Some(serde_json::json!({
                "device_id": device_id,
                "security_score": security_score
            })),
        }
    }

    /// 创建威胁告警通知
    pub fn threat_alert(
        device_id: String,
        threat_id: String,
        threat_type: String,
        severity: String,
        message: String,
    ) -> Self {
        let notification_severity = match severity.as_str() {
            "HIGH" => NotificationSeverity::High,
            "MEDIUM" => NotificationSeverity::Medium,
            _ => NotificationSeverity::Low,
        };

        Self {
            notification_type: NotificationType::ThreatAlert,
            severity: notification_severity,
            title: "威胁检测告警".to_string(),
            message,
            device_id: Some(device_id.clone()),
            threat_id: Some(threat_id.clone()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: Some(serde_json::json!({
                "device_id": device_id,
                "threat_id": threat_id,
                "threat_type": threat_type,
                "severity": severity
            })),
        }
    }

    /// 创建密钥预警通知
    pub fn key_warning(
        device_id: String,
        remaining_count: i32,
        total_count: i32,
        message: String,
    ) -> Self {
        Self {
            notification_type: NotificationType::KeyWarning,
            severity: NotificationSeverity::Medium,
            title: "密钥预警".to_string(),
            message,
            device_id: Some(device_id.clone()),
            threat_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: Some(serde_json::json!({
                "device_id": device_id,
                "remaining_count": remaining_count,
                "total_count": total_count,
                "percentage": (remaining_count as f64 / total_count as f64 * 100.0) as i32
            })),
        }
    }

    /// 创建设备状态变更通知
    pub fn device_status_change(
        device_id: String,
        old_status: String,
        new_status: String,
        reason: Option<String>,
    ) -> Self {
        Self {
            notification_type: NotificationType::DeviceStatusChange,
            severity: NotificationSeverity::Info,
            title: "设备状态变更".to_string(),
            message: format!(
                "设备 {} 状态从 {} 变更为 {}{}",
                device_id,
                old_status,
                new_status,
                reason.as_ref().map(|r| format!("，原因：{}", r)).unwrap_or_default()
            ),
            device_id: Some(device_id.clone()),
            threat_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: Some(serde_json::json!({
                "device_id": device_id,
                "old_status": old_status,
                "new_status": new_status,
                "reason": reason
            })),
        }
    }

    /// 创建系统告警通知
    pub fn system_alert(
        severity: NotificationSeverity,
        title: String,
        message: String,
        data: Option<serde_json::Value>,
    ) -> Self {
        Self {
            notification_type: NotificationType::SystemAlert,
            severity,
            title,
            message,
            device_id: None,
            threat_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            data,
        }
    }
}

/// 通知服务
pub struct NotificationService {
    pool: ConnectionPool,
}

impl NotificationService {
    /// 创建新的通知服务
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }

    /// 发送通知
    pub async fn send_notification(&self, notification: Notification) {
        info!(
            "Sending notification: type={:?}, severity={:?}, device_id={:?}",
            notification.notification_type, notification.severity, notification.device_id
        );

        match serde_json::to_string(&notification) {
            Ok(json) => {
                let message = Message::Text(json);
                broadcast_message(&self.pool, message).await;
            }
            Err(e) => {
                error!("Failed to serialize notification: {}", e);
            }
        }
    }

    /// 发送安全告警
    pub async fn send_security_alert(
        &self,
        device_id: String,
        security_score: i32,
        message: String,
    ) {
        let notification = Notification::security_alert(device_id, security_score, message);
        self.send_notification(notification).await;
    }

    /// 发送威胁告警
    pub async fn send_threat_alert(
        &self,
        device_id: String,
        threat_id: String,
        threat_type: String,
        severity: String,
        message: String,
    ) {
        let notification = Notification::threat_alert(
            device_id,
            threat_id,
            threat_type,
            severity,
            message,
        );
        self.send_notification(notification).await;
    }

    /// 发送密钥预警
    pub async fn send_key_warning(
        &self,
        device_id: String,
        remaining_count: i32,
        total_count: i32,
        message: String,
    ) {
        let notification = Notification::key_warning(
            device_id,
            remaining_count,
            total_count,
            message,
        );
        self.send_notification(notification).await;
    }

    /// 发送设备状态变更通知
    pub async fn send_device_status_change(
        &self,
        device_id: String,
        old_status: String,
        new_status: String,
        reason: Option<String>,
    ) {
        let notification = Notification::device_status_change(
            device_id,
            old_status,
            new_status,
            reason,
        );
        self.send_notification(notification).await;
    }

    /// 发送系统告警
    pub async fn send_system_alert(
        &self,
        severity: NotificationSeverity,
        title: String,
        message: String,
        data: Option<serde_json::Value>,
    ) {
        let notification = Notification::system_alert(severity, title, message, data);
        self.send_notification(notification).await;
    }

    /// 获取当前连接数
    pub async fn get_connection_count(&self) -> usize {
        super::connection::get_connection_count(&self.pool).await
    }
}
