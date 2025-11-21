use sqlx::SqlitePool;
use crate::models::{AuditLog, OperationResult};
use crate::utils::error::AppError;

/// 审计日志Repository
#[derive(Clone)]
pub struct AuditLogRepository {
    pool: SqlitePool,
}

impl AuditLogRepository {
    /// 创建新的AuditLogRepository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建审计日志
    pub async fn create(&self, log: &AuditLog) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO audit_logs (
                id, operation, operator, device_id, result,
                details, ip_address, user_agent, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            log.id,
            log.operation,
            log.operator,
            log.device_id,
            log.result,
            log.details,
            log.ip_address,
            log.user_agent,
            log.created_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 列出审计日志（支持筛选）
    pub async fn list(
        &self,
        device_id: Option<&str>,
        operator: Option<&str>,
        operation: Option<&str>,
        result: Option<OperationResult>,

        start_time: Option<&str>,
        end_time: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>, AppError> {
        let mut query = String::from(
            r#"
            SELECT 
                id, operation, operator, device_id, result,
                details, ip_address, user_agent, created_at
            FROM audit_logs
            WHERE 1=1
            "#,
        );

        if let Some(dev_id) = device_id {
            query.push_str(&format!(" AND device_id = '{}'", dev_id));
        }

        if let Some(op) = operator {
            query.push_str(&format!(" AND operator = '{}'", op));
        }

        if let Some(operation_name) = operation {
            query.push_str(&format!(" AND operation = '{}'", operation_name));
        }

        if let Some(r) = result {
            query.push_str(&format!(" AND result = '{:?}'", r));
        }

        if let Some(start) = start_time {
            query.push_str(&format!(" AND created_at >= '{}'", start));
        }

        if let Some(end) = end_time {
            query.push_str(&format!(" AND created_at <= '{}'", end));
        }

        query.push_str(" ORDER BY created_at DESC");
        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let logs = sqlx::query_as::<_, AuditLog>(&query)
            .fetch_all(&self.pool)
            .await?;

        Ok(logs)
    }

    /// 统计审计日志总数
    pub async fn count(
        &self,
        device_id: Option<&str>,
        operator: Option<&str>,
        operation: Option<&str>,
        result: Option<OperationResult>,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Result<i64, AppError> {
        let mut query = String::from("SELECT COUNT(*) as count FROM audit_logs WHERE 1=1");

        if let Some(dev_id) = device_id {
            query.push_str(&format!(" AND device_id = '{}'", dev_id));
        }

        if let Some(op) = operator {
            query.push_str(&format!(" AND operator = '{}'", op));
        }

        if let Some(operation_name) = operation {
            query.push_str(&format!(" AND operation = '{}'", operation_name));
        }

        if let Some(r) = result {
            query.push_str(&format!(" AND result = '{:?}'", r));
        }

        if let Some(start) = start_time {
            query.push_str(&format!(" AND created_at >= '{}'", start));
        }

        if let Some(end) = end_time {
            query.push_str(&format!(" AND created_at <= '{}'", end));
        }

        let result = sqlx::query_scalar::<_, i64>(&query)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    /// 根据ID查找审计日志
    pub async fn find_by_id(&self, id: &str) -> Result<Option<AuditLog>, AppError> {
        let log = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT 
                id, operation, operator, device_id,
                result as "result: _",
                details, ip_address, user_agent, created_at
            FROM audit_logs
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(log)
    }

    /// 获取设备的审计日志
    pub async fn list_by_device(
        &self,
        device_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>, AppError> {
        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT 
                id, operation, operator, device_id,
                result as "result: _",
                details, ip_address, user_agent, created_at
            FROM audit_logs
            WHERE device_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            device_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    /// 获取操作员的审计日志
    pub async fn list_by_operator(
        &self,
        operator: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>, AppError> {
        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT 
                id, operation, operator, device_id,
                result as "result: _",
                details, ip_address, user_agent, created_at
            FROM audit_logs
            WHERE operator = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            operator,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }
}
