use crate::{
    dto::{AuditLogResponse, AuditLogListResponse},
    models::{AuditLog, OperationResult},
    repositories::AuditLogRepository,
    utils::error::AppError,
};

/// 审计日志服务
#[derive(Clone)]
pub struct AuditService {
    audit_repo: AuditLogRepository,
}

impl AuditService {
    /// 创建新的审计服务
    pub fn new(audit_repo: AuditLogRepository) -> Self {
        Self { audit_repo }
    }

    /// 记录操作日志
    pub async fn log_operation(
        &self,
        operation_type: String,
        operator: String,
        result: OperationResult,
        device_id: Option<String>,
        details: Option<String>,
    ) -> Result<(), AppError> {
        tracing::debug!("Logging operation: {}", operation_type);

        let mut audit_log = AuditLog::new(operation_type, operator, result);

        if let Some(did) = device_id {
            audit_log = audit_log.with_device_id(did);
        }

        if let Some(det) = details {
            audit_log = audit_log.with_details(det);
        }

        self.audit_repo.create(&audit_log).await?;

        Ok(())
    }

    /// 记录设备注册
    pub async fn log_device_registration(
        &self,
        device_id: String,
        operator: String,
        imei: String,
        model: String,
    ) -> Result<(), AppError> {
        let details = format!("Device registered: IMEI={}, Model={}", imei, model);

        self.log_operation(
            "DEVICE_REGISTRATION".to_string(),
            operator,
            OperationResult::Success,
            Some(device_id),
            Some(details),
        )
        .await
    }

    /// 记录设备审批
    pub async fn log_device_approval(
        &self,
        device_id: String,
        operator: String,
        approved: bool,
    ) -> Result<(), AppError> {
        let operation_type = if approved {
            "DEVICE_APPROVAL"
        } else {
            "DEVICE_REJECTION"
        };

        let details = if approved {
            "Device approved and activated"
        } else {
            "Device rejected"
        };

        self.log_operation(
            operation_type.to_string(),
            operator,
            OperationResult::Success,
            Some(device_id),
            Some(details.to_string()),
        )
        .await
    }

    /// 记录PIN加密
    pub async fn log_pin_encryption(
        &self,
        device_id: String,
        operator: String,
        success: bool,
    ) -> Result<(), AppError> {
        let result = if success {
            OperationResult::Success
        } else {
            OperationResult::Failure
        };

        let details = if success {
            "PIN encrypted successfully"
        } else {
            "PIN encryption failed"
        };

        self.log_operation(
            "PIN_ENCRYPTION".to_string(),
            operator,
            result,
            Some(device_id),
            Some(details.to_string()),
        )
        .await
    }

    /// 列出审计日志
    pub async fn list_logs(
        &self,
        operation_type: Option<&str>,
        operator: Option<&str>,
        device_id: Option<&str>,
        result: Option<OperationResult>,
        start_time: Option<&str>,
        end_time: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<AuditLogListResponse, AppError> {
        tracing::debug!("Listing audit logs with limit: {}, offset: {}", limit, offset);

        let logs = self
            .audit_repo
            .list(
                device_id,
                operator,
                operation_type,
                result.clone(),
                start_time,
                end_time,
                limit,
                offset,
            )
            .await?;

        let total = self
            .audit_repo
            .count(operation_type, operator, device_id, result, start_time, end_time)
            .await?;

        let log_responses: Vec<AuditLogResponse> = logs
            .into_iter()
            .map(AuditLogResponse::from)
            .collect();

        Ok(AuditLogListResponse {
            logs: log_responses,
            total,
        })
    }

    /// 获取审计日志详情
    pub async fn get_log(&self, log_id: &str) -> Result<AuditLogResponse, AppError> {
        tracing::debug!("Getting audit log: {}", log_id);

        let log = self
            .audit_repo
            .find_by_id(log_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Audit log not found".to_string()))?;

        Ok(AuditLogResponse::from(log))
    }

    /// 获取设备的审计日志
    pub async fn get_device_logs(
        &self,
        device_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<AuditLogListResponse, AppError> {
        self.list_logs(
            None,
            None,
            Some(device_id),
            None,
            None,
            None,
            limit,
            offset,
        )
        .await
    }

    /// 获取操作员的审计日志
    pub async fn get_operator_logs(
        &self,
        operator: &str,
        limit: i64,
        offset: i64,
    ) -> Result<AuditLogListResponse, AppError> {
        self.list_logs(
            None,
            Some(operator),
            None,
            None,
            None,
            None,
            limit,
            offset,
        )
        .await
    }

    /// 获取失败操作的审计日志
    pub async fn get_failed_operations(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<AuditLogListResponse, AppError> {
        self.list_logs(
            None,
            None,
            None,
            Some(OperationResult::Failure),
            None,
            None,
            limit,
            offset,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要数据库连接
    async fn test_log_operation() {
        // 测试记录操作
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_logs() {
        // 测试列出日志
    }
}
