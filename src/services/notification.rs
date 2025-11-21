use std::sync::Arc;
use tracing::info;

use crate::api::NotificationService;

/// 通知服务包装器
/// 用于在业务逻辑层触发通知
pub struct NotificationServiceWrapper {
    notification_service: Arc<NotificationService>,
}

impl NotificationServiceWrapper {
    /// 创建新的通知服务包装器
    pub fn new(notification_service: Arc<NotificationService>) -> Self {
        Self {
            notification_service,
        }
    }

    /// 发送安全告警
    pub async fn send_security_alert(
        &self,
        device_id: String,
        security_score: i32,
        message: String,
    ) {
        info!(
            "Triggering security alert for device {}: score={}, message={}",
            device_id, security_score, message
        );
        
        self.notification_service
            .send_security_alert(device_id, security_score, message)
            .await;
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
        info!(
            "Triggering threat alert for device {}: threat_id={}, type={}, severity={}",
            device_id, threat_id, threat_type, severity
        );
        
        self.notification_service
            .send_threat_alert(device_id, threat_id, threat_type, severity, message)
            .await;
    }

    /// 发送密钥预警
    pub async fn send_key_warning(
        &self,
        device_id: String,
        remaining_count: i32,
        total_count: i32,
        message: String,
    ) {
        info!(
            "Triggering key warning for device {}: remaining={}/{}, message={}",
            device_id, remaining_count, total_count, message
        );
        
        self.notification_service
            .send_key_warning(device_id, remaining_count, total_count, message)
            .await;
    }

    /// 发送设备状态变更通知
    pub async fn send_device_status_change(
        &self,
        device_id: String,
        old_status: String,
        new_status: String,
        reason: Option<String>,
    ) {
        info!(
            "Triggering device status change notification for device {}: {} -> {}",
            device_id, old_status, new_status
        );
        
        self.notification_service
            .send_device_status_change(device_id, old_status, new_status, reason)
            .await;
    }
}
