use crate::{
    dto::{
        HealthCheckListResponse, HealthCheckRequest, HealthCheckResponse, HealthOverviewResponse,
    },
    models::{
        DeviceStatus, HealthCheck, RecommendedAction, ThreatEvent, ThreatSeverity, ThreatType,
    },
    repositories::{AuditLogRepository, DeviceRepository, HealthCheckRepository, ThreatRepository},
    security::crypto,
    services::ThreatDetectionService,
    utils::error::AppError,
};

/// 健康检查服务
#[derive(Clone)]
pub struct HealthCheckService {
    health_check_repo: HealthCheckRepository,
    device_repo: DeviceRepository,
    threat_repo: ThreatRepository,
    audit_repo: AuditLogRepository,
    threat_detection_service: ThreatDetectionService,
}

impl HealthCheckService {
    /// 创建新的健康检查服务
    pub fn new(
        health_check_repo: HealthCheckRepository,
        device_repo: DeviceRepository,
        threat_repo: ThreatRepository,
        audit_repo: AuditLogRepository,
        threat_detection_service: ThreatDetectionService,
    ) -> Self {
        Self {
            health_check_repo,
            device_repo,
            threat_repo,
            audit_repo,
            threat_detection_service,
        }
    }

    /// 提交健康检查
    pub async fn submit_health_check(
        &self,
        request: HealthCheckRequest,
        _operator: &str,
    ) -> Result<HealthCheckResponse, AppError> {
        tracing::info!("Submitting health check for device: {}", request.device_id);

        // 验证请求
        request.validate()?;

        // 验证签名
        // 验证签名
        if !request.signature.is_empty() {
            self.verify_request_signature(&request, &request.signature).await?;
        }

        // 检查设备是否存在
        let _device = self
            .device_repo
            .find_by_id(&request.device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        // 计算安全评分
        let security_score = self.calculate_security_score(
            request.root_detection,
            !request.emulator_detection, // Assume emulator detection implies TEE status
            !request.tampering_detection, // Assume tampering implies system integrity
            !request.hook_detection && !request.debugger_detection, // Assume hook/debug implies app integrity
            true, // Bootloader status not in request, assume true/locked
        );

        // 创建健康检查记录
        let health_check = HealthCheck::new(
            request.device_id.clone(),
            security_score,
            request.root_detection,
            true,                                                   // bootloader_status
            !request.tampering_detection,                           // system_integrity
            !request.hook_detection && !request.debugger_detection, // app_integrity
            !request.emulator_detection,                            // tee_status
        );

        // 保存健康检查记录
        self.health_check_repo.create(&health_check).await?;

        // 更新设备安全评分
        self.device_repo
            .update_security_score(&request.device_id, security_score)
            .await?;

        // 检测威胁类型
        let detected_threat_types = self.detect_threat_types(&health_check);

        // 【新逻辑】使用基于评分的威胁处理
        self.threat_detection_service
            .handle_health_check_threats(
                &request.device_id,
                security_score,
                detected_threat_types.clone(),
            )
            .await?;

        // 【新增】检查是否可以自动恢复设备
        let auto_recovered = self
            .threat_detection_service
            .check_auto_recovery(&request.device_id, security_score)
            .await
            .unwrap_or(false);

        if auto_recovered {
            tracing::info!("Device {} auto-recovered after health check", request.device_id);
        }

        tracing::info!(
            "Health check completed for device: {}, score: {}",
            request.device_id,
            security_score
        );

        let threat_descriptions: Vec<String> =
            detected_threat_types.iter().map(|t| format!("{:?}", t)).collect();

        Ok(HealthCheckResponse {
            check_id: health_check.id,
            device_id: request.device_id,
            security_score,
            recommended_action: self.get_recommended_action(security_score),
            threats_detected: threat_descriptions,
            checked_at: chrono::Utc::now().to_rfc3339(),
            transaction_token: None, // 将在handler层生成
        })
    }

    /// 验证请求签名
    async fn verify_request_signature(
        &self,
        request: &HealthCheckRequest,
        signature: &str,
    ) -> Result<(), AppError> {
        // 获取设备公钥
        let device = self
            .device_repo
            .find_by_id(&request.device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        // 构建待签名数据
        let data = format!(
            "{}:{}:{}:{}:{}:{}",
            request.device_id,
            request.root_detection,
            request.emulator_detection,
            request.debugger_detection,
            request.hook_detection,
            request.tampering_detection
        );

        // 验证签名
        crypto::verify_signature(&device.public_key, data.as_bytes(), signature.as_bytes())?;

        Ok(())
    }

    /// 计算安全评分
    fn calculate_security_score(
        &self,
        root_status: bool,
        bootloader_status: bool,
        system_integrity: bool,
        app_integrity: bool,
        tee_status: bool,
    ) -> i32 {
        let mut score = 100;

        // Root检测 - 扣30分
        if root_status {
            score -= 30;
        }

        // Bootloader解锁 - 扣25分
        if !bootloader_status {
            score -= 25;
        }

        // 系统完整性 - 扣20分
        if !system_integrity {
            score -= 20;
        }

        // 应用完整性 - 扣15分
        if !app_integrity {
            score -= 15;
        }

        // TEE状态 - 扣10分
        if !tee_status {
            score -= 10;
        }

        score.max(0)
    }

    /// 检测威胁类型（新方法，只返回威胁类型）
    fn detect_threat_types(&self, health_check: &HealthCheck) -> Vec<ThreatType> {
        let mut threat_types = Vec::new();

        // Root检测
        if health_check.root_status {
            threat_types.push(ThreatType::RootDetection);
        }

        // Bootloader解锁
        if !health_check.bootloader_status {
            threat_types.push(ThreatType::BootloaderUnlock);
        }

        // 系统篡改
        if !health_check.system_integrity {
            threat_types.push(ThreatType::SystemTamper);
        }

        // 应用篡改
        if !health_check.app_integrity {
            threat_types.push(ThreatType::AppTamper);
        }

        // TEE妥协
        if !health_check.tee_status {
            threat_types.push(ThreatType::TeeCompromise);
        }

        threat_types
    }

    /// 获取推荐动作（新方法）
    fn get_recommended_action(&self, score: i32) -> String {
        match score {
            0..=39 => "设备已吊销，需要重新审批".to_string(),
            40..=59 => "设备已暂停，请排查安全问题".to_string(),
            60..=89 => "建议关注设备安全状态".to_string(),
            90..=100 => "设备安全状态良好".to_string(),
            _ => "评分异常".to_string(),
        }
    }

    /// 检测威胁（已废弃，使用 detect_threat_types）
    #[allow(dead_code)]
    async fn detect_threats(
        &self,
        health_check: &HealthCheck,
    ) -> Result<Vec<ThreatEvent>, AppError> {
        let mut threats = Vec::new();

        // Root检测
        if health_check.root_status {
            threats.push(ThreatEvent::new(
                health_check.device_id.clone(),
                ThreatType::RootDetection,
                ThreatSeverity::Critical,
                "Root access detected on device".to_string(),
            ));
        }

        // Bootloader解锁
        if !health_check.bootloader_status {
            threats.push(ThreatEvent::new(
                health_check.device_id.clone(),
                ThreatType::BootloaderUnlock,
                ThreatSeverity::High,
                "Bootloader is unlocked".to_string(),
            ));
        }

        // 系统篡改
        if !health_check.system_integrity {
            threats.push(ThreatEvent::new(
                health_check.device_id.clone(),
                ThreatType::SystemTamper,
                ThreatSeverity::High,
                "System integrity compromised".to_string(),
            ));
        }

        // 应用篡改
        if !health_check.app_integrity {
            threats.push(ThreatEvent::new(
                health_check.device_id.clone(),
                ThreatType::AppTamper,
                ThreatSeverity::Medium,
                "Application integrity compromised".to_string(),
            ));
        }

        // TEE妥协
        if !health_check.tee_status {
            threats.push(ThreatEvent::new(
                health_check.device_id.clone(),
                ThreatType::TeeCompromise,
                ThreatSeverity::Critical,
                "TEE environment compromised".to_string(),
            ));
        }

        // 低安全评分
        if health_check.security_score < 50 {
            threats.push(ThreatEvent::new(
                health_check.device_id.clone(),
                ThreatType::LowSecurityScore,
                ThreatSeverity::High,
                format!("Security score too low: {}", health_check.security_score),
            ));
        }

        // 检查连续低分
        let recent_checks = self
            .health_check_repo
            .find_low_score_checks(&health_check.device_id, 50, 3)
            .await?;

        if recent_checks.len() >= 3 {
            threats.push(ThreatEvent::new(
                health_check.device_id.clone(),
                ThreatType::ConsecutiveLowScores,
                ThreatSeverity::Critical,
                "Three consecutive low security scores detected".to_string(),
            ));
        }

        Ok(threats)
    }

    /// 处理威胁
    async fn handle_threats(
        &self,
        device_id: &str,
        threats: &[ThreatEvent],
    ) -> Result<(), AppError> {
        for threat in threats {
            // 保存威胁事件
            self.threat_repo.create(threat).await?;

            // 根据威胁严重程度采取行动
            match threat.severity {
                ThreatSeverity::Critical => {
                    // 自动暂停设备
                    self.device_repo
                        .update_status(device_id, DeviceStatus::Suspended, Some("system"))
                        .await?;

                    tracing::warn!("Device {} suspended due to critical threat", device_id);
                },
                ThreatSeverity::High => {
                    // 记录警告，但不自动暂停
                    tracing::warn!("High severity threat detected on device {}", device_id);
                },
                _ => {
                    tracing::info!(
                        "Threat detected on device {}: {:?}",
                        device_id,
                        threat.threat_type
                    );
                },
            }
        }

        Ok(())
    }

    /// 执行初始健康检查
    pub async fn perform_initial_check(
        &self,
        device_id: &str,
    ) -> Result<HealthCheckResponse, AppError> {
        tracing::info!("Performing initial health check for device: {}", device_id);

        // 创建默认的健康检查请求
        let request = HealthCheckRequest {
            device_id: device_id.to_string(),
            root_detection: false,
            emulator_detection: false,
            debugger_detection: false,
            hook_detection: false,
            tampering_detection: false,
            signature: "".to_string(),
        };

        self.submit_health_check(request, "system").await
    }

    /// 列出健康检查记录
    pub async fn list_health_checks(
        &self,
        device_id: Option<&str>,
        start_time: Option<&str>,
        end_time: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<HealthCheckListResponse, AppError> {
        tracing::debug!("Listing health checks with limit: {}, offset: {}", limit, offset);

        let checks = if let Some(did) = device_id {
            self.health_check_repo
                .list_by_device(did, start_time, end_time, limit, offset)
                .await?
        } else {
            // 如果没有指定设备，返回空列表
            Vec::new()
        };

        let total = if device_id.is_some() {
            self.health_check_repo.count_by_device(device_id.unwrap()).await?
        } else {
            0
        };

        let check_responses: Vec<HealthCheckResponse> = checks
            .into_iter()
            .map(|check| HealthCheckResponse {
                check_id: check.id,
                device_id: check.device_id,
                security_score: check.security_score,
                recommended_action: check.recommended_action.to_string(),
                threats_detected: Vec::new(), // 需要单独查询
                checked_at: check.created_at,
                transaction_token: None, // 历史记录不包含令牌
            })
            .collect();

        Ok(HealthCheckListResponse { checks: check_responses, total })
    }

    /// 获取健康概览
    pub async fn get_health_overview(
        &self,
        device_id: &str,
    ) -> Result<HealthOverviewResponse, AppError> {
        tracing::debug!("Getting health overview for device: {}", device_id);

        // 获取最新的健康检查
        let latest_check = self.health_check_repo.get_latest_by_device(device_id).await?;

        // 获取活跃威胁
        let _active_threats = self.threat_repo.get_active_threats(device_id).await?;

        // 获取健康检查历史统计
        let total_checks = self.health_check_repo.count_by_device(device_id).await?;

        Ok(HealthOverviewResponse {
            device_id: device_id.to_string(),
            latest_score: latest_check.as_ref().map(|c| c.security_score).unwrap_or(0),
            average_score: 0.0, // TODO: Calculate average score
            total_checks,
            last_check_at: latest_check.map(|c| c.created_at),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calculate_security_score() {
        let pool = sqlx::SqlitePool::connect("").await.unwrap();
        let health_check_repo = HealthCheckRepository::new(pool.clone());
        let device_repo = DeviceRepository::new(pool.clone());
        let threat_repo = ThreatRepository::new(pool.clone());
        let audit_repo = AuditLogRepository::new(pool.clone());
        let threat_detection_service = ThreatDetectionService::new(
            threat_repo.clone(),
            device_repo.clone(),
            health_check_repo.clone(),
            audit_repo.clone(),
        );

        let service = HealthCheckService::new(
            health_check_repo,
            device_repo,
            threat_repo,
            audit_repo,
            threat_detection_service,
        );

        // 完美状态
        assert_eq!(service.calculate_security_score(false, true, true, true, true), 100);

        // Root检测
        assert_eq!(service.calculate_security_score(true, true, true, true, true), 70);

        // 所有问题
        assert_eq!(service.calculate_security_score(true, false, false, false, false), 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_submit_health_check() {
        // 测试提交健康检查
    }
}
