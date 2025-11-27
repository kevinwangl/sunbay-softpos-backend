use crate::{
    dto::{ThreatResponse, ThreatListResponse, ThreatStatisticsResponse},
    models::{ThreatEvent, ThreatStatus, ThreatSeverity, ThreatType, DeviceStatus, AuditLog, OperationResult},
    repositories::{ThreatRepository, DeviceRepository, HealthCheckRepository, AuditLogRepository},
    utils::error::AppError,
};

/// 威胁检测服务
#[derive(Clone)]
pub struct ThreatDetectionService {
    threat_repo: ThreatRepository,
    device_repo: DeviceRepository,
    health_check_repo: HealthCheckRepository,
    audit_repo: AuditLogRepository,
}

impl ThreatDetectionService {
    /// 创建新的威胁检测服务
    pub fn new(
        threat_repo: ThreatRepository,
        device_repo: DeviceRepository,
        health_check_repo: HealthCheckRepository,
        audit_repo: AuditLogRepository,
    ) -> Self {
        Self {
            threat_repo,
            device_repo,
            health_check_repo,
            audit_repo,
        }
    }

    /// 处理威胁
    pub async fn handle_threats(
        &self,
        device_id: &str,
        threats: &[ThreatEvent],
    ) -> Result<(), AppError> {
        tracing::info!("Handling {} threats for device: {}", threats.len(), device_id);

        for threat in threats {
            // 保存威胁事件
            self.threat_repo.create(threat).await?;

            // 评估威胁严重程度并采取行动
            let action = self.assess_threat_severity(threat).await?;

            match action {
                ThreatAction::Suspend => {
                    self.device_repo
                        .update_status(device_id, DeviceStatus::Suspended, Some("system"))
                        .await?;

                    tracing::warn!("Device {} suspended due to threat: {:?}", device_id, threat.threat_type);
                }
                ThreatAction::Revoke => {
                    self.device_repo
                        .update_status(device_id, DeviceStatus::Revoked, Some("system"))
                        .await?;

                    tracing::error!("Device {} revoked due to threat: {:?}", device_id, threat.threat_type);
                }
                ThreatAction::Monitor => {
                    tracing::warn!("Monitoring threat on device {}: {:?}", device_id, threat.threat_type);
                }
                ThreatAction::None => {
                    tracing::info!("Low priority threat on device {}: {:?}", device_id, threat.threat_type);
                }
            }

            // 记录审计日志
            let audit_log = AuditLog::new(
                "THREAT_DETECTED".to_string(),
                "system".to_string(),
                OperationResult::Success,
            )
            .with_device_id(device_id.to_string())
            .with_details(format!(
                "Threat detected: type={:?}, severity={:?}, action={:?}",
                threat.threat_type, threat.severity, action
            ));

            self.audit_repo.create(&audit_log).await?;
        }

        Ok(())
    }

    /// 评估威胁严重程度
    async fn assess_threat_severity(&self, threat: &ThreatEvent) -> Result<ThreatAction, AppError> {
        let action = match threat.severity {
            ThreatSeverity::Critical => {
                // 关键威胁：自动暂停或吊销
                match threat.threat_type {
                    ThreatType::RootDetection | ThreatType::TeeCompromise => ThreatAction::Suspend,
                    ThreatType::ConsecutiveLowScores => ThreatAction::Revoke,
                    _ => ThreatAction::Suspend,
                }
            }
            ThreatSeverity::High => {
                // 高危威胁：暂停设备
                ThreatAction::Suspend
            }
            ThreatSeverity::Medium => {
                // 中等威胁：监控
                ThreatAction::Monitor
            }
            ThreatSeverity::Low => {
                // 低危威胁：记录
                ThreatAction::None
            }
        };

        Ok(action)
    }

    /// 检查连续低分
    pub async fn check_consecutive_low_scores(&self, device_id: &str) -> Result<bool, AppError> {
        tracing::debug!("Checking consecutive low scores for device: {}", device_id);

        let recent_checks = self
            .health_check_repo
            .find_low_score_checks(device_id, 50, 3)
            .await?;

        Ok(recent_checks.len() >= 3)
    }

    /// 列出威胁
    pub async fn list_threats(
        &self,
        device_id: Option<&str>,
        status: Option<ThreatStatus>,
        severity: Option<ThreatSeverity>,
        threat_type: Option<ThreatType>,
        limit: i64,
        offset: i64,
    ) -> Result<ThreatListResponse, AppError> {
        tracing::debug!("Listing threats with limit: {}, offset: {}", limit, offset);

        let threats = self
            .threat_repo
            .list(device_id, status, severity, threat_type, limit, offset)
            .await?;

        let total = self.threat_repo.count_by_status(status).await?;

        let threat_responses: Vec<ThreatResponse> = threats
            .into_iter()
            .map(ThreatResponse::from)
            .collect();

        Ok(ThreatListResponse {
            threats: threat_responses,
            total,
        })
    }

    /// 解决威胁
    pub async fn resolve_threat(
        &self,
        threat_id: &str,
        operator: &str,
        resolution_notes: Option<String>,
    ) -> Result<(), AppError> {
        tracing::info!("Resolving threat: {}", threat_id);

        // 检查威胁是否存在
        let threat = self
            .threat_repo
            .find_by_id(threat_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Threat not found".to_string()))?;

        if threat.status == ThreatStatus::Resolved {
            return Err(AppError::BadRequest("Threat already resolved".to_string()));
        }

        // 更新威胁状态
        self.threat_repo
            .update_status(threat_id, ThreatStatus::Resolved, Some(operator))
            .await?;

        // 记录审计日志
        let details = if let Some(notes) = resolution_notes {
            format!("Threat resolved: {}", notes)
        } else {
            "Threat resolved".to_string()
        };

        let audit_log = AuditLog::new(
            "THREAT_RESOLVED".to_string(),
            operator.to_string(),
            OperationResult::Success,
        )
        .with_device_id(threat.device_id)
        .with_details(details);

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("Threat resolved: {}", threat_id);

        Ok(())
    }

    /// 获取威胁统计
    pub async fn get_threat_statistics(&self) -> Result<ThreatStatisticsResponse, AppError> {
        tracing::debug!("Getting threat statistics");

        let stats = self.threat_repo.get_statistics().await?;

        Ok(ThreatStatisticsResponse {
            total: stats.total,
            active: stats.active,
            resolved: stats.resolved,
            critical: stats.by_severity.critical,
            high: stats.by_severity.high,
            medium: stats.by_severity.medium,
            low: stats.by_severity.low,
        })
    }

    /// 获取设备的活跃威胁
    pub async fn get_device_active_threats(&self, device_id: &str) -> Result<Vec<ThreatResponse>, AppError> {
        tracing::debug!("Getting active threats for device: {}", device_id);

        let threats = self.threat_repo.get_active_threats(device_id).await?;

        let threat_responses: Vec<ThreatResponse> = threats
            .into_iter()
            .map(ThreatResponse::from)
            .collect();

        Ok(threat_responses)
    }

    /// 上报威胁（由设备端调用）
    pub async fn report_threat(
        &self,
        device_id: &str,
        threat_type: ThreatType,
        severity: ThreatSeverity,
        description: String,
    ) -> Result<ThreatResponse, AppError> {
        tracing::info!("Device {} reporting threat: {:?}", device_id, threat_type);

        // 验证设备是否存在
        let device = self
            .device_repo
            .find_by_id(device_id)
            .await?
            .ok_or_else(|| AppError::DeviceNotFound)?;

        // 创建威胁事件
        let threat = ThreatEvent::new(
            device_id.to_string(),
            threat_type,
            severity,
            description,
        );

        // 保存威胁事件
        self.threat_repo.create(&threat).await?;

        // 评估威胁并采取行动
        let action = self.assess_threat_severity(&threat).await?;

        match action {
            ThreatAction::Suspend => {
                self.device_repo
                    .update_status(device_id, DeviceStatus::Suspended, Some("system"))
                    .await?;
                tracing::warn!("Device {} suspended due to reported threat", device_id);
            }
            ThreatAction::Revoke => {
                self.device_repo
                    .update_status(device_id, DeviceStatus::Revoked, Some("system"))
                    .await?;
                tracing::error!("Device {} revoked due to reported threat", device_id);
            }
            _ => {}
        }

        // 记录审计日志
        let audit_log = AuditLog::new(
            "THREAT_REPORTED".to_string(),
            device_id.to_string(),
            OperationResult::Success,
        )
        .with_device_id(device_id.to_string())
        .with_details(format!(
            "Threat reported by device: type={:?}, severity={:?}",
            threat_type, severity
        ));

        self.audit_repo.create(&audit_log).await?;

        Ok(ThreatResponse::from(threat))
    }

    /// 批量解决威胁
    pub async fn bulk_resolve_threats(
        &self,
        threat_ids: Vec<String>,
        operator: &str,
    ) -> Result<i32, AppError> {
        tracing::info!("Bulk resolving {} threats", threat_ids.len());

        let mut resolved_count = 0;

        for threat_id in threat_ids {
            match self.resolve_threat(&threat_id, operator, None).await {
                Ok(_) => resolved_count += 1,
                Err(e) => {
                    tracing::warn!("Failed to resolve threat {}: {}", threat_id, e);
                }
            }
        }

        Ok(resolved_count)
    }
}

/// 威胁处理动作
#[derive(Debug, Clone, Copy)]
enum ThreatAction {
    None,
    Monitor,
    Suspend,
    Revoke,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要数据库连接
    async fn test_handle_threats() {
        // 测试处理威胁
    }

    #[tokio::test]
    #[ignore]
    async fn test_assess_threat_severity() {
        // 测试评估威胁严重程度
    }
}
