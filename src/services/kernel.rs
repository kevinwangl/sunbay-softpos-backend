use std::path::PathBuf;

use tokio::fs;
use uuid::Uuid;

use crate::{
    models::{Kernel, KernelStatus},
    repositories::KernelRepository,
    utils::error::AppError,
};

#[derive(Clone)]
pub struct KernelService {
    kernel_repo: KernelRepository,
    storage_path: String,
}

impl KernelService {
    pub fn new(kernel_repo: KernelRepository, storage_path: String) -> Self {
        Self {
            kernel_repo,
            storage_path,
        }
    }

    /// 上传内核文件
    pub async fn upload_kernel(
        &self,
        version: &str,
        file_bytes: Vec<u8>,
        filename: &str,
    ) -> Result<Kernel, AppError> {
        // 验证版本号格式
        if !version.starts_with('v') {
            return Err(AppError::BadRequest("Version must start with 'v'".to_string()));
        }

        // 检查版本是否已存在
        if let Some(_) = self.kernel_repo.find_by_version(version).await? {
            return Err(AppError::BadRequest(format!("Kernel version {} already exists", version)));
        }

        // 生成唯一ID
        let id = Uuid::new_v4().to_string();
        let kernel_dir = PathBuf::from(&self.storage_path).join("kernels").join(&id);

        // 创建目录
        fs::create_dir_all(&kernel_dir).await.map_err(|e| {
            AppError::InternalWithMessage(format!("Failed to create directory: {}", e))
        })?;

        // 保存文件
        let file_path = kernel_dir.join(filename);
        fs::write(&file_path, &file_bytes)
            .await
            .map_err(|e| AppError::InternalWithMessage(format!("Failed to write file: {}", e)))?;

        // 计算哈希
        let file_hash =
            hex::encode(ring::digest::digest(&ring::digest::SHA256, &file_bytes).as_ref());

        // 创建内核记录
        let now = chrono::Utc::now().to_rfc3339();
        let kernel = Kernel {
            id,
            version: version.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            file_hash,
            file_size: file_bytes.len() as i64,
            status: KernelStatus::Draft.to_string(),
            created_at: now.clone(),
            updated_at: now,
        };

        let kernel = self.kernel_repo.create(&kernel).await?;

        tracing::info!("Kernel {} uploaded successfully", version);

        Ok(kernel)
    }

    /// 列出所有内核
    pub async fn list_kernels(&self, status: Option<&str>) -> Result<Vec<Kernel>, AppError> {
        if let Some(status) = status {
            self.kernel_repo.list_by_status(status).await
        } else {
            self.kernel_repo.list_all().await
        }
    }

    /// 获取内核详情
    pub async fn get_kernel(&self, version: &str) -> Result<Kernel, AppError> {
        self.kernel_repo
            .find_by_version(version)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Kernel version {} not found", version)))
    }

    /// 下载内核文件
    pub async fn download_kernel(&self, version: &str) -> Result<Vec<u8>, AppError> {
        let kernel = self.get_kernel(version).await?;

        let file_bytes = fs::read(&kernel.file_path).await.map_err(|e| {
            AppError::InternalWithMessage(format!("Failed to read kernel file: {}", e))
        })?;

        Ok(file_bytes)
    }

    /// 发布内核版本
    pub async fn publish_kernel(&self, version: &str) -> Result<(), AppError> {
        // 确保内核存在
        self.get_kernel(version).await?;

        // 更新状态为stable
        self.kernel_repo
            .update_status(version, &KernelStatus::Stable.to_string())
            .await?;

        tracing::info!("Kernel {} published", version);

        Ok(())
    }

    /// 删除内核
    pub async fn delete_kernel(&self, version: &str) -> Result<(), AppError> {
        let kernel = self.get_kernel(version).await?;

        // 删除文件
        let file_path = PathBuf::from(&kernel.file_path);
        if let Some(parent) = file_path.parent() {
            fs::remove_dir_all(parent).await.map_err(|e| {
                AppError::InternalWithMessage(format!("Failed to delete kernel files: {}", e))
            })?;
        }

        // 删除数据库记录
        self.kernel_repo.delete(version).await?;

        tracing::info!("Kernel {} deleted", version);

        Ok(())
    }
}
