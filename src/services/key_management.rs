use crate::{
    dto::{
        InjectKeyRequest, InjectKeyResponse, UpdateKeyRequest, UpdateKeyResponse,
        KeyStatusResponse, EncryptPinRequest, EncryptPinResponse,
    },
    models::{Device, DeviceStatus, AuditLog, OperationResult},
    repositories::{DeviceRepository, AuditLogRepository},
    security::{DukptKeyDerivation, crypto},
    infrastructure::HsmClient,
    utils::error::AppError,
};

/// 密钥管理服务
#[derive(Clone)]
pub struct KeyManagementService {
    device_repo: DeviceRepository,
    audit_repo: AuditLogRepository,
    dukpt: DukptKeyDerivation,
    hsm_client: Option<HsmClient>,
}

impl KeyManagementService {
    /// 创建新的密钥管理服务
    pub fn new(
        device_repo: DeviceRepository,
        audit_repo: AuditLogRepository,
        dukpt: DukptKeyDerivation,
        hsm_client: Option<HsmClient>,
    ) -> Self {
        Self {
            device_repo,
            audit_repo,
            dukpt,
            hsm_client,
        }
    }

    /// 注入密钥
    pub async fn inject_key(
        &self,
        request: InjectKeyRequest,
        operator: &str,
    ) -> Result<InjectKeyResponse, AppError> {
        tracing::info!("Injecting key for device: {}", request.device_id);

        // 验证请求
        request.validate()?;

        // 检查设备是否存在且已审批
        let device = self
            .device_repo
            .find_by_id(&request.device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        if device.status != DeviceStatus::Active.as_str() {
            return Err(AppError::BadRequest(
                "Device must be in active status for key injection".to_string(),
            ));
        }

        // 检查是否已经注入过密钥
        if device.ipek_injected_at.is_some() {
            return Err(AppError::BadRequest(
                "Key has already been injected for this device".to_string(),
            ));
        }

        let ksn = &device.current_ksn;

        // 派生IPEK
        let ipek = if let Some(ref hsm_client) = self.hsm_client {
            // 使用HSM派生IPEK
            hsm_client.derive_ipek(ksn, &request.device_id).await?
        } else {
            // 使用本地DUKPT派生IPEK
            self.dukpt.derive_ipek(ksn)?
        };

        // 使用设备公钥加密IPEK
        let public_key_pem = String::from_utf8(device.public_key.clone())
            .map_err(|_| AppError::Internal)?;
        let encrypted_ipek = crypto::encrypt_with_public_key(&public_key_pem, &ipek)?;
        let encrypted_ipek_b64 = crypto::base64_encode(&encrypted_ipek);

        // 更新设备密钥信息
        let now = chrono::Utc::now().to_rfc3339();
        self.device_repo
            .update_key_info(
                &request.device_id,
                ksn,
                Some(&now),
                Some(1000), // 重置剩余次数
                Some(1000), // 默认最大使用次数
            )
            .await?;

        // 记录审计日志
        let audit_log = AuditLog::new(
            "KEY_INJECTION".to_string(),
            operator.to_string(),
            OperationResult::Success,
        )
        .with_device_id(request.device_id.clone())
        .with_details("IPEK injected successfully".to_string());

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("Key injected successfully for device: {}", request.device_id);

        Ok(InjectKeyResponse {
            device_id: request.device_id,
            encrypted_ipek: encrypted_ipek_b64,
            ksn: ksn.clone(),
            injected_at: now,
            message: "Key injected successfully".to_string(),
        })
    }

    /// 更新密钥
    pub async fn update_key(
        &self,
        request: UpdateKeyRequest,
        operator: &str,
    ) -> Result<UpdateKeyResponse, AppError> {
        tracing::info!("Updating key for device: {}", request.device_id);

        // 验证请求
        request.validate()?;

        // 检查设备是否存在且已注入密钥
        let device = self
            .device_repo
            .find_by_id(&request.device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        if device.status != DeviceStatus::Active.as_str() {
            return Err(AppError::BadRequest(
                "Device must be in active status for key update".to_string(),
            ));
        }

        if device.ipek_injected_at.is_none() {
            return Err(AppError::BadRequest(
                "Key must be injected before update".to_string(),
            ));
        }

        let current_ksn = &device.current_ksn;

        // 生成新的KSN
        let new_ksn = self.dukpt.increment_ksn(current_ksn)?;

        // 派生新的IPEK
        let new_ipek = if let Some(ref hsm_client) = self.hsm_client {
            hsm_client.derive_ipek(&new_ksn, &request.device_id).await?
        } else {
            self.dukpt.derive_ipek(&new_ksn)?
        };

        // 使用设备公钥加密新IPEK
        let public_key_pem = String::from_utf8(device.public_key.clone())
            .map_err(|_| AppError::Internal)?;
        let encrypted_ipek = crypto::encrypt_with_public_key(&public_key_pem, &new_ipek)?;
        let encrypted_ipek_b64 = crypto::base64_encode(&encrypted_ipek);

        // 更新设备密钥信息
        let now = chrono::Utc::now().to_rfc3339();
        self.device_repo
            .update_key_info(
                &request.device_id,
                &new_ksn,
                None,
                Some(device.key_total_count), // 重置剩余次数为总次数
                Some(device.key_total_count),
            )
            .await?;

        // 记录审计日志
        let audit_log = AuditLog::new(
            "KEY_UPDATE".to_string(),
            operator.to_string(),
            OperationResult::Success,
        )
        .with_device_id(request.device_id.clone())
        .with_details(format!("Key updated from KSN {} to {}", current_ksn, new_ksn));

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("Key updated successfully for device: {}", request.device_id);

        Ok(UpdateKeyResponse {
            device_id: request.device_id,
            new_ksn,
            encrypted_ipek: encrypted_ipek_b64,
            updated_at: now,
            message: "Key updated successfully".to_string(),
        })
    }

    /// 获取密钥状态
    pub async fn get_key_status(
        &self,
        device_id: &str,
    ) -> Result<KeyStatusResponse, AppError> {
        tracing::debug!("Getting key status for device: {}", device_id);

        // 检查设备是否存在
        let device = self
            .device_repo
            .find_by_id(device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        // 检查是否已注入密钥
        if device.ipek_injected_at.is_none() {
            return Ok(KeyStatusResponse {
                device_id: device_id.to_string(),
                current_ksn: "".to_string(),
                remaining_count: 0,
                status: "INACTIVE".to_string(),
                last_updated: device.updated_at,
                next_update_required: None,
            });
        }

        let current_ksn = device.current_ksn.clone();
        let max_usage = device.key_total_count;
        let remaining_count = device.key_remaining_count;
        
        // Determine status
        let status = if remaining_count <= 0 {
            "EXPIRED".to_string()
        } else if remaining_count < (max_usage / 10) {
            "NEAR_EXPIRY".to_string()
        } else {
            "ACTIVE".to_string()
        };

        Ok(KeyStatusResponse {
            device_id: device_id.to_string(),
            current_ksn,
            remaining_count,
            status,
            last_updated: device.updated_at,
            next_update_required: None,
        })
    }

    /// 加密PIN
    pub async fn encrypt_pin(
        &self,
        request: EncryptPinRequest,
        operator: &str,
    ) -> Result<EncryptPinResponse, AppError> {
        tracing::debug!("Encrypting PIN for device: {}", request.device_id);

        // 验证请求
        request.validate()?;

        // 检查设备是否存在且已注入密钥
        let device = self
            .device_repo
            .find_by_id(&request.device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        if device.status != DeviceStatus::Active.as_str() {
            return Err(AppError::BadRequest(
                "Device must be in active status".to_string(),
            ));
        }

        if device.ipek_injected_at.is_none() {
            return Err(AppError::BadRequest(
                "Key must be injected before PIN encryption".to_string(),
            ));
        }

        let ksn = &device.current_ksn;

        // 派生IPEK和Working Key
        let ipek = if let Some(ref hsm_client) = self.hsm_client {
            hsm_client.derive_ipek(ksn, &request.device_id).await?
        } else {
            self.dukpt.derive_ipek(ksn)?
        };

        let working_key = if let Some(ref hsm_client) = self.hsm_client {
            hsm_client.derive_working_key(&ipek, ksn).await?
        } else {
            self.dukpt.derive_working_key(&ipek, ksn)?
        };

        // 加密PIN Block
        let encrypted_pin_block = self.dukpt.encrypt_pin_block(&request.pin, &working_key)?;
        let encrypted_pin_block_hex = hex::encode(&encrypted_pin_block);

        // 递增密钥使用次数
        self.device_repo.decrement_key_count(&request.device_id).await?;

        // 记录审计日志
        let audit_log = AuditLog::new(
            "PIN_ENCRYPTION".to_string(),
            operator.to_string(),
            OperationResult::Success,
        )
        .with_device_id(request.device_id.clone())
        .with_details("PIN encrypted successfully".to_string());

        self.audit_repo.create(&audit_log).await?;

        let now = chrono::Utc::now().to_rfc3339();

        tracing::debug!("PIN encrypted successfully for device: {}", request.device_id);

        Ok(EncryptPinResponse {
            encrypted_pin_block: encrypted_pin_block_hex,
            ksn: ksn.clone(),
            device_id: request.device_id,
            encrypted_at: now,
        })
    }

    /// 检查密钥是否需要更新
    pub async fn check_key_update_needed(&self, device_id: &str) -> Result<bool, AppError> {
        let key_status = self.get_key_status(device_id).await?;
        Ok(key_status.status == "NEAR_EXPIRY" || key_status.status == "EXPIRED")
    }

    /// 批量检查需要更新密钥的设备
    pub async fn get_devices_needing_key_update(&self) -> Result<Vec<String>, AppError> {
        tracing::debug!("Getting devices needing key update");

        // 获取所有活跃设备
        let devices = self
            .device_repo
            .list(Some(DeviceStatus::Active), None, 1000, 0)
            .await?;

        let mut devices_needing_update = Vec::new();

        for device in devices {
            let max_usage = device.key_total_count;
            let remaining = device.key_remaining_count;
            if remaining < (max_usage / 10) {
                devices_needing_update.push(device.id);
            }
        }

        Ok(devices_needing_update)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要数据库连接
    async fn test_inject_key() {
        // 测试密钥注入
    }

    #[tokio::test]
    #[ignore]
    async fn test_key_lifecycle() {
        // 测试密钥完整生命周期：注入 -> 使用 -> 更新
    }
}
