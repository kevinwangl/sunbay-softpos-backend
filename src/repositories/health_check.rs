use crate::{
    models::HealthCheck,
    utils::error::AppError,
};
use sqlx::SqlitePool;

/// 健康检查Repository
#[derive(Clone)]
pub struct HealthCheckRepository {
    pool: SqlitePool,
}

impl HealthCheckRepository {
    /// 创建新的健康检查Repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建健康检查记录
    pub async fn create(&self, health_check: &HealthCheck) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO health_checks (
                id, device_id, security_score, root_status, bootloader_status,
                system_integrity, app_integrity, tee_status, recommended_action,
                details, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            health_check.id,
            health_check.device_id,
            health_check.security_score,
            health_check.root_status,
            health_check.bootloader_status,
            health_check.system_integrity,
            health_check.app_integrity,
            health_check.tee_status,
            health_check.recommended_action,
            health_check.details,
            health_check.created_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 根据设备ID查找健康检查记录
    pub async fn find_by_device_id(&self, device_id: &str) -> Result<Vec<HealthCheck>, AppError> {
        let records = sqlx::query_as!(
            HealthCheck,
            r#"
            SELECT 
                id, device_id, 
                security_score as "security_score: i32", 
                root_status as "root_status: bool",
                bootloader_status as "bootloader_status: bool",
                system_integrity as "system_integrity: bool",
                app_integrity as "app_integrity: bool",
                tee_status as "tee_status: bool",
                recommended_action as "recommended_action: _",
                details, created_at
            FROM health_checks
            WHERE device_id = ?
            ORDER BY created_at DESC
            "#,
            device_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// 列出设备的健康检查记录（支持时间范围）
    pub async fn list_by_device(
        &self,
        device_id: &str,
        start_time: Option<&str>,
        end_time: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<HealthCheck>, AppError> {
        let mut query = String::from(
            r#"
            SELECT 
                id, device_id, security_score, root_status, bootloader_status,
                system_integrity, app_integrity, tee_status, recommended_action,
                details, created_at
            FROM health_checks
            WHERE device_id = ?
            "#
        );

        let mut params: Vec<String> = vec![device_id.to_string()];

        if let Some(start) = start_time {
            query.push_str(" AND created_at >= ?");
            params.push(start.to_string());
        }

        if let Some(end) = end_time {
            query.push_str(" AND created_at <= ?");
            params.push(end.to_string());
        }

        query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        let records = sqlx::query_as::<_, HealthCheck>(&query)
            .bind(device_id)
            .bind(start_time.unwrap_or(""))
            .bind(end_time.unwrap_or(""))
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(records)
    }

    /// 获取设备最新的健康检查记录
    pub async fn get_latest_by_device(&self, device_id: &str) -> Result<Option<HealthCheck>, AppError> {
        let record = sqlx::query_as!(
            HealthCheck,
            r#"
            SELECT 
                id, device_id, 
                security_score as "security_score: i32",
                root_status as "root_status: bool",
                bootloader_status as "bootloader_status: bool",
                system_integrity as "system_integrity: bool",
                app_integrity as "app_integrity: bool",
                tee_status as "tee_status: bool",
                recommended_action as "recommended_action: _",
                details, created_at
            FROM health_checks
            WHERE device_id = ?
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            device_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    /// 统计设备的健康检查次数
    pub async fn count_by_device(&self, device_id: &str) -> Result<i64, AppError> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM health_checks
            WHERE device_id = ?
            "#,
            device_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count as i64)
    }

    /// 获取低安全评分的健康检查记录
    pub async fn find_low_score_checks(
        &self,
        device_id: &str,
        threshold: i32,
        limit: i64,
    ) -> Result<Vec<HealthCheck>, AppError> {
        let records = sqlx::query_as!(
            HealthCheck,
            r#"
            SELECT 
                id, device_id, 
                security_score as "security_score: i32",
                root_status as "root_status: bool",
                bootloader_status as "bootloader_status: bool",
                system_integrity as "system_integrity: bool",
                app_integrity as "app_integrity: bool",
                tee_status as "tee_status: bool",
                recommended_action as "recommended_action: _",
                details, created_at
            FROM health_checks
            WHERE device_id = ? AND security_score < ?
            ORDER BY created_at DESC
            LIMIT ?
            "#,
            device_id,
            threshold,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要数据库连接
    async fn test_create_health_check() {
        // 测试创建健康检查记录
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_latest_by_device() {
        // 测试获取最新健康检查
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_low_score_checks() {
        // 测试查找低分记录
    }
}
