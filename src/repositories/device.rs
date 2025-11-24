use sqlx::SqlitePool;
use crate::models::{Device, DeviceStatus};
use crate::utils::error::AppError;

/// 设备Repository
#[derive(Clone)]
pub struct DeviceRepository {
    pool: SqlitePool,
}

impl DeviceRepository {
    /// 创建新的DeviceRepository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建设备
    pub async fn create(&self, device: &Device) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO devices (
                id, imei, model, os_version, tee_type, device_mode, public_key,
                status, security_score, current_ksn, registered_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            device.id,
            device.imei,
            device.model,
            device.os_version,
            device.tee_type,
            device.device_mode,
            device.public_key,
            device.status,
            device.security_score,
            device.current_ksn,
            device.registered_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 根据ID查找设备
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Device>, AppError> {
        let device = sqlx::query_as!(
            Device,
            r#"
            SELECT 
                id, imei, model, os_version, tee_type, device_mode, public_key,
                status, merchant_id, merchant_name, 
                security_score as "security_score: i32",
                current_ksn, ipek_injected_at, 
                key_remaining_count as "key_remaining_count: i32", 
                key_total_count as "key_total_count: i32",
                registered_at, approved_at, approved_by, last_active_at, updated_at
            FROM devices
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(device)
    }

    /// 检查IMEI是否存在
    pub async fn exists_by_imei(&self, imei: &str) -> Result<bool, AppError> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM devices WHERE imei = ?
            "#,
            imei
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count > 0)
    }

    /// 列出设备（支持筛选、搜索、排序、分页）
    pub async fn list(
        &self,
        status: Option<DeviceStatus>,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Device>, AppError> {
        let mut query = String::from(
            r#"
            SELECT 
                id, imei, model, os_version, 
                tee_type, device_mode, public_key,
                status, merchant_id, merchant_name,
                security_score,
                current_ksn, ipek_injected_at,
                key_remaining_count, key_total_count,
                registered_at, approved_at, approved_by,
                last_active_at, updated_at
            FROM devices
            WHERE 1=1
            "#,
        );

        // 添加状态筛选
        if let Some(s) = status {
            query.push_str(&format!(" AND status = '{:?}'", s));
        }

        // 添加搜索条件
        if let Some(search_term) = search {
            query.push_str(&format!(
                " AND (imei LIKE '%{}%' OR model LIKE '%{}%')",
                search_term, search_term
            ));
        }

        // 添加排序和分页
        query.push_str(" ORDER BY registered_at DESC");
        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let devices = sqlx::query_as::<_, Device>(&query)
            .fetch_all(&self.pool)
            .await?;

        Ok(devices)
    }

    /// 统计设备总数
    pub async fn count(
        &self,
        status: Option<DeviceStatus>,
        search: Option<&str>,
    ) -> Result<i64, AppError> {
        let mut query = String::from("SELECT COUNT(*) as count FROM devices WHERE 1=1");

        if let Some(s) = status {
            query.push_str(&format!(" AND status = '{:?}'", s));
        }

        if let Some(search_term) = search {
            query.push_str(&format!(
                " AND (imei LIKE '%{}%' OR model LIKE '%{}%')",
                search_term, search_term
            ));
        }

        let result = sqlx::query_scalar::<_, i64>(&query)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    /// 更新设备状态
    pub async fn update_status(
        &self,
        id: &str,
        status: DeviceStatus,
        approved_by: Option<&str>,
    ) -> Result<(), AppError> {
        let now = chrono::Utc::now().to_rfc3339();

        let status_str = status.as_str();
        
        if status == DeviceStatus::Active {
            // 审批通过
            sqlx::query!(
                r#"
                UPDATE devices
                SET status = ?, approved_at = ?, approved_by = ?
                WHERE id = ?
                "#,
                status_str,
                now,
                approved_by,
                id
            )
            .execute(&self.pool)
            .await?;
        } else {
            // 其他状态变更
            sqlx::query!(
                r#"
                UPDATE devices
                SET status = ?
                WHERE id = ?
                "#,
                status_str,
                id
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// 更新安全评分
    pub async fn update_security_score(&self, id: &str, score: i32) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE devices
            SET security_score = ?
            WHERE id = ?
            "#,
            score,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 更新KSN
    pub async fn update_ksn(&self, id: &str, ksn: &str) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE devices
            SET current_ksn = ?
            WHERE id = ?
            "#,
            ksn,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 更新密钥信息
    pub async fn update_key_info(
        &self,
        id: &str,
        ksn: &str,
        injected_at: Option<&str>,
        key_remaining_count: Option<i32>,
        key_total_count: Option<i32>,
    ) -> Result<(), AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query!(
            r#"
            UPDATE devices
            SET current_ksn = ?,
                ipek_injected_at = COALESCE(?, ipek_injected_at),
                key_remaining_count = COALESCE(?, key_remaining_count),
                key_total_count = COALESCE(?, key_total_count),
                updated_at = ?
            WHERE id = ?
            "#,
            ksn,
            injected_at,
            key_remaining_count,
            key_total_count,
            now,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 递减密钥使用次数
    pub async fn decrement_key_count(&self, id: &str) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE devices
            SET key_remaining_count = key_remaining_count - 1
            WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 获取设备统计信息
    pub async fn get_statistics(&self) -> Result<DeviceStatistics, AppError> {
        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM devices")
            .fetch_one(&self.pool)
            .await?;

        let active = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM devices WHERE status = 'Active'",
        )
        .fetch_one(&self.pool)
        .await?;

        let pending = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM devices WHERE status = 'Pending'",
        )
        .fetch_one(&self.pool)
        .await?;

        let suspended = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM devices WHERE status = 'Suspended'",
        )
        .fetch_one(&self.pool)
        .await?;

        let revoked = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM devices WHERE status = 'Revoked'",
        )
        .fetch_one(&self.pool)
        .await?;

        let avg_score = sqlx::query_scalar::<_, f64>(
            "SELECT AVG(security_score) FROM devices WHERE status = 'Active'",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DeviceStatistics {
            total,
            active,
            pending,
            suspended,
            revoked,
            average_security_score: avg_score,
        })
    }
}

/// 设备统计信息
#[derive(Debug, Clone)]
pub struct DeviceStatistics {
    pub total: i64,
    pub active: i64,
    pub pending: i64,
    pub suspended: i64,
    pub revoked: i64,
    pub average_security_score: f64,
}
