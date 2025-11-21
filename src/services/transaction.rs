use crate::{
    dto::{
        AttestTransactionRequest, AttestTransactionResponse, ProcessTransactionRequest,
        ProcessTransactionResponse, TransactionResponse, TransactionListResponse,
        AttestPinpadRequest, AttestPinpadResponse,
    },
    models::{
        Transaction, TransactionStatus, TransactionType, DeviceStatus, DeviceMode,
        AuditLog, OperationResult,
    },
    repositories::{TransactionRepository, DeviceRepository, AuditLogRepository},
    security::{DukptKeyDerivation, crypto},
    infrastructure::HsmClient,
    utils::error::AppError,
};

/// 交易服务
#[derive(Clone)]
pub struct TransactionService {
    transaction_repo: TransactionRepository,
    device_repo: DeviceRepository,
    audit_repo: AuditLogRepository,
    dukpt: DukptKeyDerivation,
    hsm_client: Option<HsmClient>,
}

impl TransactionService {
    /// 创建新的交易服务
    pub fn new(
        transaction_repo: TransactionRepository,
        device_repo: DeviceRepository,
        audit_repo: AuditLogRepository,
        dukpt: DukptKeyDerivation,
        hsm_client: Option<HsmClient>,
    ) -> Self {
        Self {
            transaction_repo,
            device_repo,
            audit_repo,
            dukpt,
            hsm_client,
        }
    }

    /// 交易鉴证（SoftPOS模式）
    pub async fn attest_transaction(
        &self,
        request: AttestTransactionRequest,
        operator: &str,
    ) -> Result<AttestTransactionResponse, AppError> {
        tracing::info!("Attesting transaction for device: {}", request.device_id);

        // 验证请求
        request.validate()?;

        // 检查设备是否存在且为活跃状态
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

        if DeviceMode::from_str(&device.device_mode) != Some(DeviceMode::FullPos) {
            return Err(AppError::BadRequest(
                "Device must be in SoftPOS mode for transaction attestation".to_string(),
            ));
        }

        // 检查设备是否已注入密钥
        if device.ipek_injected_at.is_none() {
            return Err(AppError::BadRequest(
                "Device key must be injected before transaction".to_string(),
            ));
        }

        // 生成交易令牌（有效期15分钟）
        let transaction_token = crypto::generate_random_hex(32);
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);

        // 记录审计日志
        let audit_log = AuditLog::new(
            "TRANSACTION_ATTESTATION".to_string(),
            operator.to_string(),
            OperationResult::Success,
        )
        .with_device_id(request.device_id.clone())
        .with_details(format!(
            "Transaction attested: amount={}, currency={}",
            request.amount, request.currency
        ));

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("Transaction attested successfully for device: {}", request.device_id);

        Ok(AttestTransactionResponse {
            transaction_token,
            device_id: request.device_id,
            expires_at: expires_at.to_rfc3339(),
            message: "Transaction attestation successful".to_string(),
        })
    }

    /// 处理交易
    pub async fn process_transaction(
        &self,
        request: ProcessTransactionRequest,
        operator: &str,
    ) -> Result<ProcessTransactionResponse, AppError> {
        tracing::info!("Processing transaction for device: {}", request.device_id);

        // 验证请求
        request.validate()?;

        // 检查设备是否存在且为活跃状态
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

        // 验证KSN
        let device_ksn = &device.current_ksn;

        if &request.ksn != device_ksn {
            return Err(AppError::BadRequest("Invalid KSN".to_string()));
        }

        // 创建交易记录
        let mut transaction = Transaction::new(
            request.device_id.clone(),
            request.transaction_type.clone(),
            request.amount,
            request.currency.clone(),
            request.ksn.clone(),
        );

        transaction.encrypted_pin_block = Some(request.encrypted_pin_block);
        transaction.card_number_masked = request.card_number_masked;

        // 模拟交易处理（在实际环境中，这里会调用支付网关）
        let (status, auth_code, response_code, response_message) = 
            self.simulate_transaction_processing(&transaction).await?;

        transaction.status = status.clone();
        transaction.authorization_code = auth_code.clone();
        transaction.response_code = response_code.clone();
        transaction.response_message = response_message.clone();

        // 保存交易记录
        self.transaction_repo.create(&transaction).await?;

        // 如果交易成功，递增密钥使用次数
        if status == TransactionStatus::Approved {
            self.device_repo.decrement_key_count(&request.device_id).await?;
        }

        // 记录审计日志
        let audit_result = if status == TransactionStatus::Approved {
            OperationResult::Success
        } else {
            OperationResult::Failure
        };

        let audit_log = AuditLog::new(
            "TRANSACTION_PROCESSING".to_string(),
            operator.to_string(),
            audit_result,
        )
        .with_device_id(request.device_id.clone())
        .with_details(format!(
            "Transaction processed: type={:?}, amount={}, status={:?}",
            request.transaction_type, request.amount, status
        ));

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("Transaction processed: {} - {:?}", transaction.id, status);

        Ok(ProcessTransactionResponse {
            transaction_id: transaction.id,
            status,
            authorization_code: auth_code,
            message: response_message.unwrap_or_else(|| "Transaction processed".to_string()),
        })
    }

    /// PINPad设备鉴证
    pub async fn attest_pinpad(
        &self,
        request: AttestPinpadRequest,
        operator: &str,
    ) -> Result<AttestPinpadResponse, AppError> {
        tracing::info!("Attesting PINPad device: {}", request.device_id);

        // 验证请求
        request.validate()?;

        // 检查设备是否存在且为活跃状态
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

        if DeviceMode::from_str(&device.device_mode) != Some(DeviceMode::PinPad) {
            return Err(AppError::BadRequest(
                "Device must be in PINPad mode".to_string(),
            ));
        }

        // 检查设备是否已注入密钥
        if device.ipek_injected_at.is_none() {
            return Err(AppError::BadRequest(
                "Device key must be injected before attestation".to_string(),
            ));
        }

        // 生成鉴证令牌（有效期30分钟）
        let attestation_token = crypto::generate_random_hex(32);
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);

        // 记录审计日志
        let audit_log = AuditLog::new(
            "PINPAD_ATTESTATION".to_string(),
            operator.to_string(),
            OperationResult::Success,
        )
        .with_device_id(request.device_id.clone())
        .with_details("PINPad device attested successfully".to_string());

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("PINPad attested successfully: {}", request.device_id);

        Ok(AttestPinpadResponse {
            attestation_token,
            device_id: request.device_id,
            expires_at: expires_at.to_rfc3339(),
            message: "PINPad attestation successful".to_string(),
        })
    }

    /// 获取交易详情
    pub async fn get_transaction(&self, transaction_id: &str) -> Result<TransactionResponse, AppError> {
        tracing::debug!("Getting transaction: {}", transaction_id);

        let transaction = self
            .transaction_repo
            .find_by_id(transaction_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Transaction not found".to_string()))?;

        Ok(TransactionResponse::from(transaction))
    }

    /// 列出交易
    pub async fn list_transactions(
        &self,
        device_id: Option<&str>,
        status: Option<TransactionStatus>,
        transaction_type: Option<TransactionType>,
        limit: i64,
        offset: i64,
    ) -> Result<TransactionListResponse, AppError> {
        tracing::debug!("Listing transactions with limit: {}, offset: {}", limit, offset);

        let transactions = self
            .transaction_repo
            .list(device_id, status.clone(), transaction_type.clone(), limit, offset)
            .await?;

        let total = self
            .transaction_repo
            .count(device_id, status, transaction_type)
            .await?;

        let transaction_responses: Vec<TransactionResponse> = transactions
            .into_iter()
            .map(TransactionResponse::from)
            .collect();

        Ok(TransactionListResponse {
            transactions: transaction_responses,
            total,
        })
    }

    /// 获取设备交易统计
    pub async fn get_device_transaction_stats(
        &self,
        device_id: &str,
    ) -> Result<crate::repositories::TransactionStats, AppError> {
        tracing::debug!("Getting transaction stats for device: {}", device_id);

        // 检查设备是否存在
        self.device_repo
            .find_by_id(device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        let stats = self.transaction_repo.get_device_transaction_stats(device_id).await?;
        Ok(stats)
    }

    /// 模拟交易处理
    /// 
    /// 在实际环境中，这里会调用真实的支付网关或处理器
    async fn simulate_transaction_processing(
        &self,
        transaction: &Transaction,
    ) -> Result<(TransactionStatus, Option<String>, Option<String>, Option<String>), AppError> {
        // 模拟处理延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // 简单的模拟逻辑
        let success_rate = match transaction.transaction_type {
            TransactionType::Payment => 0.95,
            TransactionType::PreAuth => 0.98,
            TransactionType::Refund => 0.99,
            TransactionType::Void => 0.99,
            TransactionType::Capture => 0.97,
        };

        // 使用交易金额作为随机种子
        let random_factor = (transaction.amount % 100) as f64 / 100.0;

        if random_factor < success_rate {
            // 交易成功
            let auth_code = crypto::generate_random_hex(6).to_uppercase();
            Ok((
                TransactionStatus::Approved,
                Some(auth_code),
                Some("00".to_string()),
                Some("Transaction approved".to_string()),
            ))
        } else {
            // 交易失败
            let decline_codes = vec![
                ("05", "Do not honor"),
                ("14", "Invalid card number"),
                ("51", "Insufficient funds"),
                ("54", "Expired card"),
                ("61", "Exceeds withdrawal amount limit"),
            ];

            let code_index = (transaction.amount % decline_codes.len() as i64) as usize;
            let (response_code, response_message) = decline_codes[code_index];

            Ok((
                TransactionStatus::Declined,
                None,
                Some(response_code.to_string()),
                Some(response_message.to_string()),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要数据库连接
    async fn test_attest_transaction() {
        // 测试交易鉴证
    }

    #[tokio::test]
    #[ignore]
    async fn test_process_transaction() {
        // 测试交易处理
    }
}
