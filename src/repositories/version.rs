use sqlx::SqlitePool;
use crate::models::{SdkVersion, VersionStatus};
use crate::utils::error::AppError;

/// SDK版本Repository
#[derive(Clone)]
pub struct VersionRepository {
    pool: SqlitePool,
}

impl VersionRepository {
    /// 创建新的VersionRepository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建版本
    pub async fn create(&self, version: &SdkVersion) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO sdk_versions (
                id, version, update_type, status, download_url,
                checksum, file_size, release_notes, min_os_version,
                target_devices, distribution_strategy, created_at, released_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            version.id,
            version.version,
            version.update_type,
            version.status,
            version.download_url,
            version.checksum,
            version.file_size,
            version.release_notes,
            version.min_os_version,
            version.target_devices,
            version.distribution_strategy,
            version.created_at,
            version.released_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 根据ID查找版本
    pub async fn find_by_id(&self, id: &str) -> Result<Option<SdkVersion>, AppError> {
        let version = sqlx::query_as!(
            SdkVersion,
            r#"
            SELECT 
                id, version,
                update_type as "update_type: _",
                status as "status: _",
                download_url, checksum, file_size, release_notes,
                min_os_version, target_devices, distribution_strategy,
                created_at, released_at
            FROM sdk_versions
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(version)
    }

    /// 根据版本号查找
    pub async fn find_by_version(&self, version: &str) -> Result<Option<SdkVersion>, AppError> {
        let sdk_version = sqlx::query_as!(
            SdkVersion,
            r#"
            SELECT 
                id, version,
                update_type as "update_type: _",
                status as "status: _",
                download_url, checksum, file_size, release_notes,
                min_os_version, target_devices, distribution_strategy,
                created_at, released_at
            FROM sdk_versions
            WHERE version = ?
            "#,
            version
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(sdk_version)
    }

    /// 列出版本
    pub async fn list(
        &self,
        status: Option<VersionStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SdkVersion>, AppError> {
        let mut query = String::from(
            r#"
            SELECT 
                id, version, update_type, status, download_url,
                checksum, file_size, release_notes, min_os_version,
                target_devices, distribution_strategy, created_at, released_at
            FROM sdk_versions
            WHERE 1=1
            "#,
        );

        if let Some(s) = status {
            query.push_str(&format!(" AND status = '{:?}'", s));
        }

        query.push_str(" ORDER BY created_at DESC");
        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let versions = sqlx::query_as::<_, SdkVersion>(&query)
            .fetch_all(&self.pool)
            .await?;

        Ok(versions)
    }

    /// 统计版本总数
    pub async fn count(&self, status: Option<VersionStatus>) -> Result<i64, AppError> {
        let mut query = String::from("SELECT COUNT(*) as count FROM sdk_versions WHERE 1=1");

        if let Some(s) = status {
            query.push_str(&format!(" AND status = '{:?}'", s));
        }

        let result = sqlx::query_scalar::<_, i64>(&query)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    /// 更新版本
    pub async fn update(&self, version: &SdkVersion) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE sdk_versions
            SET status = ?,
                download_url = ?,
                checksum = ?,
                file_size = ?,
                release_notes = ?,
                min_os_version = ?,
                target_devices = ?,
                distribution_strategy = ?,
                released_at = ?
            WHERE id = ?
            "#,
            version.status,
            version.download_url,
            version.checksum,
            version.file_size,
            version.release_notes,
            version.min_os_version,
            version.target_devices,
            version.distribution_strategy,
            version.released_at,
            version.id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 更新版本状态
    pub async fn update_status(&self, id: &str, status: VersionStatus) -> Result<(), AppError> {
        let released_at = if status == VersionStatus::Released {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        };

        sqlx::query!(
            r#"
            UPDATE sdk_versions
            SET status = ?, released_at = ?
            WHERE id = ?
            "#,
            status,
            released_at,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 获取最新的已发布版本
    pub async fn get_latest_released(&self) -> Result<Option<SdkVersion>, AppError> {
        let version = sqlx::query_as!(
            SdkVersion,
            r#"
            SELECT 
                id, version,
                update_type as "update_type: _",
                status as "status: _",
                download_url, checksum, file_size, release_notes,
                min_os_version, target_devices, distribution_strategy,
                created_at, released_at
            FROM sdk_versions
            WHERE status = 'Released'
            ORDER BY released_at DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(version)
    }
}
