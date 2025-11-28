use crate::{
    dto::{
        ApproveDeviceRequest, DeviceListResponse, DeviceResponse, RegisterDeviceRequest,
        RegisterDeviceResponse, RejectDeviceRequest,
    },
    infrastructure::HsmClient,
    models::{AuditLog, Device, DeviceStatus, OperationResult},
    repositories::{AuditLogRepository, DeviceRepository},
    security::DukptKeyDerivation,
    utils::error::AppError,
};

/// 设备服务
#[derive(Clone)]
pub struct DeviceService {
    device_repo: DeviceRepository,
    audit_repo: AuditLogRepository,
    dukpt: DukptKeyDerivation,
    hsm_client: Option<HsmClient>,
}

impl DeviceService {
    /// 创建新的设备服务
    pub fn new(
        device_repo: DeviceRepository,
        audit_repo: AuditLogRepository,
        dukpt: DukptKeyDerivation,
        hsm_client: Option<HsmClient>,
    ) -> Self {
        Self { device_repo, audit_repo, dukpt, hsm_client }
    }

    /// 注册设备
    pub async fn register_device(
        &self,
        request: RegisterDeviceRequest,
        operator: &str,
    ) -> Result<RegisterDeviceResponse, AppError> {
        tracing::info!("Registering device with IMEI: {}", request.imei);

        // 验证请求
        request.validate()?;

        // 检查IMEI是否已存在
        if self.device_repo.exists_by_imei(&request.imei).await? {
            return Err(AppError::BadRequest("IMEI already exists".to_string()));
        }

        // 生成初始KSN
        let ksn = self.dukpt.generate_initial_ksn(&request.imei)?;

        // 创建设备
        let mut device = Device::new(
            request.imei.clone(),
            request.model,
            request.os_version,
            request.tee_type,
            request.public_key.into_bytes(),
            request.device_mode,
            request.nfc_present,
        );

        // 更新设备的KSN
        device.current_ksn = ksn.clone();

        // 保存设备
        self.device_repo.create(&device).await?;

        // 记录审计日志
        let audit_log = AuditLog::new(
            "DEVICE_REGISTRATION".to_string(),
            operator.to_string(),
            OperationResult::Success,
        )
        .with_device_id(device.id.clone())
        .with_details(format!("Device registered: IMEI={}, Model={}", request.imei, device.model));

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("Device registered successfully: {}", device.id);

        Ok(RegisterDeviceResponse {
            device_id: device.id,
            ksn,
            status: DeviceStatus::Pending,
            message: "Device registered successfully. Awaiting approval.".to_string(),
        })
    }

    /// 审批设备
    pub async fn approve_device(&self, request: ApproveDeviceRequest) -> Result<(), AppError> {
        tracing::info!("Approving device: {}", request.device_id);

        // 验证请求
        request.validate()?;

        // 检查设备是否存在
        let device = self
            .device_repo
            .find_by_id(&request.device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        // 检查设备状态
        if device.status != DeviceStatus::Pending.as_str() {
            return Err(AppError::BadRequest("Device is not in pending status".to_string()));
        }

        // 更新设备状态为Active
        self.device_repo
            .update_status(&request.device_id, DeviceStatus::Active, Some(&request.operator))
            .await?;

        // 记录审计日志
        let audit_log = AuditLog::new(
            "DEVICE_APPROVAL".to_string(),
            request.operator,
            OperationResult::Success,
        )
        .with_device_id(request.device_id.clone())
        .with_details("Device approved and activated".to_string());

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("Device approved successfully: {}", request.device_id);

        Ok(())
    }

    /// 拒绝设备
    pub async fn reject_device(&self, request: RejectDeviceRequest) -> Result<(), AppError> {
        tracing::info!("Rejecting device: {}", request.device_id);

        // 验证请求
        request.validate()?;

        // 检查设备是否存在
        let device = self
            .device_repo
            .find_by_id(&request.device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        // 检查设备状态
        if device.status != DeviceStatus::Pending.as_str() {
            return Err(AppError::BadRequest("Device is not in pending status".to_string()));
        }

        // 更新设备状态为Revoked
        self.device_repo
            .update_status(&request.device_id, DeviceStatus::Revoked, Some(&request.operator))
            .await?;

        // 记录审计日志
        let audit_log = AuditLog::new(
            "DEVICE_REJECTION".to_string(),
            request.operator,
            OperationResult::Success,
        )
        .with_device_id(request.device_id.clone())
        .with_details(format!("Device rejected: {}", request.reason));

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("Device rejected successfully: {}", request.device_id);

        Ok(())
    }

    /// 获取设备详情
    pub async fn get_device(&self, device_id: &str) -> Result<DeviceResponse, AppError> {
        tracing::debug!("Getting device: {}", device_id);

        let device = self
            .device_repo
            .find_by_id(device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        Ok(DeviceResponse::from(device))
    }

    /// 列出设备
    pub async fn list_devices(
        &self,
        status: Option<DeviceStatus>,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<DeviceListResponse, AppError> {
        tracing::debug!("Listing devices with limit: {}, offset: {}", limit, offset);

        let devices = self.device_repo.list(status, search, limit, offset).await?;

        let total = self.device_repo.count(status, search).await?;

        let device_responses: Vec<DeviceResponse> =
            devices.into_iter().map(DeviceResponse::from).collect();

        Ok(DeviceListResponse { devices: device_responses, total })
    }

    /// 暂停设备
    pub async fn suspend_device(
        &self,
        device_id: &str,
        operator: &str,
        reason: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Suspending device: {}", device_id);

        // 检查设备是否存在且为Active状态
        let device = self
            .device_repo
            .find_by_id(device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        if device.status != DeviceStatus::Active.as_str() {
            return Err(AppError::BadRequest("Device is not in active status".to_string()));
        }

        // 更新设备状态为Suspended
        self.device_repo
            .update_status(device_id, DeviceStatus::Suspended, Some(operator))
            .await?;

        // 记录审计日志
        let audit_log = AuditLog::new(
            "DEVICE_SUSPENSION".to_string(),
            operator.to_string(),
            OperationResult::Success,
        )
        .with_device_id(device_id.to_string())
        .with_details(format!("Device suspended: {}", reason));

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("Device suspended successfully: {}", device_id);

        Ok(())
    }

    /// 恢复设备
    pub async fn resume_device(&self, device_id: &str, operator: &str) -> Result<(), AppError> {
        tracing::info!("Resuming device: {}", device_id);

        // 检查设备是否存在且为Suspended状态
        let device = self
            .device_repo
            .find_by_id(device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        if device.status != DeviceStatus::Suspended.as_str() {
            return Err(AppError::BadRequest("Device is not in suspended status".to_string()));
        }

        // 更新设备状态为Active
        self.device_repo
            .update_status(device_id, DeviceStatus::Active, Some(operator))
            .await?;

        // 记录审计日志
        let audit_log = AuditLog::new(
            "DEVICE_RESUMPTION".to_string(),
            operator.to_string(),
            OperationResult::Success,
        )
        .with_device_id(device_id.to_string())
        .with_details("Device resumed and activated".to_string());

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("Device resumed successfully: {}", device_id);

        Ok(())
    }

    /// 吊销设备
    pub async fn revoke_device(
        &self,
        device_id: &str,
        operator: &str,
        reason: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Revoking device: {}", device_id);

        // 检查设备是否存在
        let device = self
            .device_repo
            .find_by_id(device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        // 只有Active或Suspended状态的设备可以被吊销
        let current_status =
            DeviceStatus::from_str(&device.status).ok_or_else(|| AppError::Internal)?;

        if !matches!(current_status, DeviceStatus::Active | DeviceStatus::Suspended) {
            return Err(AppError::BadRequest(
                "Device cannot be revoked in current status".to_string(),
            ));
        }

        // 更新设备状态为Revoked
        self.device_repo
            .update_status(device_id, DeviceStatus::Revoked, Some(operator))
            .await?;

        // 记录审计日志
        let audit_log = AuditLog::new(
            "DEVICE_REVOCATION".to_string(),
            operator.to_string(),
            OperationResult::Success,
        )
        .with_device_id(device_id.to_string())
        .with_details(format!("Device revoked: {}", reason));

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("Device revoked successfully: {}", device_id);

        Ok(())
    }

    /// 更新设备安全评分
    pub async fn update_security_score(
        &self,
        device_id: &str,
        score: i32,
        operator: &str,
    ) -> Result<(), AppError> {
        tracing::debug!("Updating security score for device: {}, score: {}", device_id, score);

        // 验证评分范围
        if !(0..=100).contains(&score) {
            return Err(AppError::BadRequest(
                "Security score must be between 0 and 100".to_string(),
            ));
        }

        // 检查设备是否存在
        let device = self
            .device_repo
            .find_by_id(device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        // 更新安全评分
        self.device_repo.update_security_score(device_id, score).await?;

        // 记录审计日志
        let audit_log = AuditLog::new(
            "SECURITY_SCORE_UPDATE".to_string(),
            operator.to_string(),
            OperationResult::Success,
        )
        .with_device_id(device_id.to_string())
        .with_details(format!(
            "Security score updated from {} to {}",
            device.security_score, score
        ));

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("Security score updated for device: {}", device_id);

        Ok(())
    }

    /// 获取设备统计信息
    pub async fn get_device_statistics(
        &self,
    ) -> Result<crate::repositories::DeviceStatistics, AppError> {
        tracing::debug!("Getting device statistics");
        let stats = self.device_repo.get_statistics().await?;
        Ok(stats)
    }

    /// 鉴证PINPad设备
    pub async fn attest_pinpad_device(
        &self,
        _request: crate::dto::request::AttestPinpadRequest,
    ) -> Result<crate::dto::response::AttestPinpadResponse, AppError> {
        // TODO: Implement PINPad attestation
        Ok(crate::dto::response::AttestPinpadResponse {
            device_id: "placeholder".to_string(),
            attestation_token: "token_placeholder".to_string(),
            expires_at: "2025-12-31T23:59:59Z".to_string(),
            message: "PINPad attestation simulated".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要数据库连接
    async fn test_register_device() {
        // 测试设备注册
    }

    #[tokio::test]
    #[ignore]
    async fn test_device_lifecycle() {
        // 测试设备完整生命周期：注册 -> 审批 -> 暂停 -> 恢复 -> 吊销
    }
}
