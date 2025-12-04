use sqlx::SqlitePool;

use crate::{models::Kernel, utils::error::AppError};

#[derive(Clone)]
pub struct KernelRepository {
    pool: SqlitePool,
}

impl KernelRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
        }
    }

    /// 创建新内核记录
    pub async fn create(&self, kernel: &Kernel) -> Result<Kernel, AppError> {
        let kernel = sqlx::query_as::<_, Kernel>(
            r#"
            INSERT INTO kernels (id, version, file_path, file_hash, file_size, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&kernel.id)
        .bind(&kernel.version)
        .bind(&kernel.file_path)
        .bind(&kernel.file_hash)
        .bind(kernel.file_size)
        .bind(&kernel.status)
        .bind(&kernel.created_at)
        .bind(&kernel.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(kernel)
    }

    /// 根据版本号查找内核
    pub async fn find_by_version(&self, version: &str) -> Result<Option<Kernel>, AppError> {
        let kernel = sqlx::query_as::<_, Kernel>(
            r#"
            SELECT * FROM kernels WHERE version = ?
            "#,
        )
        .bind(version)
        .fetch_optional(&self.pool)
        .await?;

        Ok(kernel)
    }

    /// 列出所有内核
    pub async fn list_all(&self) -> Result<Vec<Kernel>, AppError> {
        let kernels = sqlx::query_as::<_, Kernel>(
            r#"
            SELECT * FROM kernels ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(kernels)
    }

    /// 根据状态列出内核
    pub async fn list_by_status(&self, status: &str) -> Result<Vec<Kernel>, AppError> {
        let kernels = sqlx::query_as::<_, Kernel>(
            r#"
            SELECT * FROM kernels WHERE status = ? ORDER BY created_at DESC
            "#,
        )
        .bind(status)
        .fetch_all(&self.pool)
        .await?;

        Ok(kernels)
    }

    /// 更新内核状态
    pub async fn update_status(&self, version: &str, status: &str) -> Result<(), AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            UPDATE kernels SET status = ?, updated_at = ? WHERE version = ?
            "#,
        )
        .bind(status)
        .bind(now)
        .bind(version)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除内核
    pub async fn delete(&self, version: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM kernels WHERE version = ?
            "#,
        )
        .bind(version)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
