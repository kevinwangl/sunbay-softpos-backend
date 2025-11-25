use crate::{
    dto::{
        CreateVersionRequest, CreateVersionResponse, VersionResponse, VersionListResponse,
        UpdateVersionRequest,
    },
    models::{SdkVersion, VersionStatus, UpdateType, AuditLog, OperationResult},
    repositories::{VersionRepository, DeviceRepository, AuditLogRepository},
    utils::error::AppError,
};

/// 版本管理服务
#[derive(Clone)]
pub struct VersionService {
    version_repo: VersionRepository,
    device_repo: DeviceRepository,
    audit_repo: AuditLogRepository,
}

impl VersionService {
    /// 创建新的版本服务
    pub fn new(
        version_repo: VersionRepository,
        device_repo: DeviceRepository,
        audit_repo: AuditLogRepository,
    ) -> Self {
        Self {
            version_repo,
            device_repo,
            audit_repo,
        }
    }

    /// 创建版本
    pub async fn create_version(
        &self,
        request: CreateVersionRequest,
        operator: &str,
    ) -> Result<CreateVersionResponse, AppError> {
        tracing::info!("Creating version: {}", request.version);

        // 验证请求
        request.validate()?;

        // 验证语义化版本号
        self.validate_semantic_version(&request.version)?;

        // 检查版本是否已存在
        if let Some(_) = self.version_repo.find_by_version(&request.version).await? {
            return Err(AppError::BadRequest(
                format!("Version {} already exists", request.version),
            ));
        }

        // 创建版本
        let version = SdkVersion::new(
            request.version.clone(),
            request.update_type,
            request.download_url,
            request.checksum,
            request.file_size,
            request.release_notes,
        );

        // 保存版本
        self.version_repo.create(&version).await?;

        // 记录审计日志
        let audit_log = AuditLog::new(
            "VERSION_CREATED".to_string(),
            operator.to_string(),
            OperationResult::Success,
        )
        .with_details(format!("Version {} created", request.version));

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("Version created successfully: {}", version.id);

        Ok(CreateVersionResponse {
            id: version.id,
            version: version.version,
            status: version.status,
            message: "Version created successfully".to_string(),
        })
    }

    /// 验证语义化版本号
    fn validate_semantic_version(&self, version: &str) -> Result<(), AppError> {
        // 简单的语义化版本验证：major.minor.patch
        let parts: Vec<&str> = version.split('.').collect();

        if parts.len() != 3 {
            return Err(AppError::InvalidVersionFormat(
                "Version must be in format: major.minor.patch".to_string(),
            ));
        }

        for part in parts {
            if part.parse::<u32>().is_err() {
                return Err(AppError::InvalidVersionFormat(
                    "Version parts must be numbers".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// 列出版本
    pub async fn list_versions(
        &self,
        status: Option<VersionStatus>,
        _update_type: Option<UpdateType>,
        limit: i64,
        offset: i64,
    ) -> Result<VersionListResponse, AppError> {
        tracing::debug!("Listing versions with limit: {}, offset: {}", limit, offset);

        let versions = self
            .version_repo
            .list(status.clone(), limit, offset)
            .await?;

        let total = self.version_repo.count(status).await?;

        let version_responses: Vec<VersionResponse> = versions
            .into_iter()
            .map(VersionResponse::from)
            .collect();

        Ok(VersionListResponse {
            versions: version_responses,
            total,
        })
    }

    /// 获取可用版本
    pub async fn get_available_version(
        &self,
        device_id: &str,
    ) -> Result<Option<VersionResponse>, AppError> {
        tracing::debug!(
            "Getting available version for device: {}",
            device_id
        );

        // 获取设备信息
        let device = self
            .device_repo
            .find_by_id(device_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        // 获取所有可用版本
        let versions = self
            .version_repo
            .list(Some(VersionStatus::Released), 100, 0)
            .await?;

        // 筛选适用的版本
        let mut applicable_versions: Vec<SdkVersion> = versions
            .into_iter()
            .filter(|v| {
                // 检查版本是否比当前版本新
                self.is_newer_version(&v.version, &device.os_version)
                    // 检查设备是否在目标列表中
                    && v.target_devices.as_ref().map_or(true, |targets| {
                        targets.contains(&device_id.to_string())
                    })
                    // 检查OS版本要求
                    && v.min_os_version.as_ref().map_or(true, |min_os| {
                        self.meets_os_requirement(&device.os_version, min_os)
                    })
            })
            .collect();

        // 按版本号排序，返回最新的
        applicable_versions.sort_by(|a, b| {
            self.compare_versions(&b.version, &a.version)
        });

        Ok(applicable_versions.first().map(|v| VersionResponse::from(v.clone())))
    }

    /// 比较版本号
    fn is_newer_version(&self, new_version: &str, current_version: &str) -> bool {
        self.compare_versions(new_version, current_version) == std::cmp::Ordering::Greater
    }

    /// 比较两个版本号
    fn compare_versions(&self, v1: &str, v2: &str) -> std::cmp::Ordering {
        let parts1: Vec<u32> = v1.split('.').filter_map(|s| s.parse().ok()).collect();
        let parts2: Vec<u32> = v2.split('.').filter_map(|s| s.parse().ok()).collect();

        for i in 0..3 {
            let p1 = parts1.get(i).unwrap_or(&0);
            let p2 = parts2.get(i).unwrap_or(&0);

            match p1.cmp(p2) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }

        std::cmp::Ordering::Equal
    }

    /// 检查OS版本要求
    fn meets_os_requirement(&self, device_os: &str, min_os: &str) -> bool {
        self.compare_versions(device_os, min_os) != std::cmp::Ordering::Less
    }

    /// 更新版本
    pub async fn update_version(
        &self,
        version_id: &str,
        request: UpdateVersionRequest,
        operator: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Updating version: {}", version_id);

        // 验证请求
        request.validate()?;

        // 检查版本是否存在
        let version = self
            .version_repo
            .find_by_id(version_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Version not found".to_string()))?;

        // 更新版本字段
        let mut version = version;
        if let Some(status) = request.status {
            version.status = status;
        }
        if let Some(notes) = request.release_notes {
            version.release_notes = notes;
        }
        if let Some(url) = request.download_url {
            version.download_url = url;
        }

        // 保存更新
        self.version_repo.update(&version).await?;

        // 记录审计日志
        let audit_log = AuditLog::new(
            "VERSION_UPDATED".to_string(),
            operator.to_string(),
            OperationResult::Success,
        )
        .with_details(format!("Version {} updated", version.version));

        self.audit_repo.create(&audit_log).await?;

        tracing::info!("Version updated successfully: {}", version_id);

        Ok(())
    }

    /// 获取版本详情
    pub async fn get_version(&self, version_id: &str) -> Result<VersionResponse, AppError> {
        tracing::debug!("Getting version: {}", version_id);

        let version = self
            .version_repo
            .find_by_id(version_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Version not found".to_string()))?;

        Ok(VersionResponse::from(version))
    }

    /// 获取版本统计
    pub async fn get_version_statistics(&self) -> Result<VersionStatistics, AppError> {
        tracing::debug!("Getting version statistics");

        let total = self.version_repo.count(None).await?;
        let available = self
            .version_repo
            .count(Some(VersionStatus::Released))
            .await?;
        let deprecated = self
            .version_repo
            .count(Some(VersionStatus::Deprecated))
            .await?;

        Ok(VersionStatistics {
            total,
            available,
            deprecated,
        })
    }

    /// 获取过时设备列表
    pub async fn get_outdated_devices(&self) -> Result<Vec<String>, AppError> {
        tracing::debug!("Getting outdated devices");

        // 获取最新版本
        let latest_version = self
            .version_repo
            .list(Some(VersionStatus::Released), 1, 0)
            .await?
            .first()
            .map(|v| v.version.clone())
            .ok_or_else(|| AppError::NotFound("No released version found".to_string()))?;

        tracing::debug!("Latest version: {}", latest_version);

        // 获取所有活跃设备
        let devices = self
            .device_repo
            .list(Some(crate::models::DeviceStatus::Active), None, 1000, 0)
            .await?;

        // 筛选版本低于最新版本的设备
        let outdated: Vec<String> = devices
            .into_iter()
            .filter(|d| {
                // 如果设备没有OS版本，或者版本低于最新版本，则认为是过时的
                // 注意：这里简化了逻辑，实际应该比较版本号
                d.os_version != latest_version
            })
            .map(|d| d.id)
            .collect();

        Ok(outdated)
    }

    /// 创建推送任务
    pub async fn create_push_task(
        &self,
        _request: crate::dto::CreatePushTaskRequest,
        _operator: &str,
    ) -> Result<String, AppError> {
        // TODO: Implement push task creation
        Ok("task_id_placeholder".to_string())
    }

    /// 列出推送任务
    pub async fn list_push_tasks(
        &self,
        _version_id: Option<&str>,
        _status: Option<&str>,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<String>, AppError> {
        // TODO: Implement push task listing
        Ok(vec![])
    }

    /// 获取推送任务详情
    pub async fn get_push_task(&self, _task_id: &str) -> Result<String, AppError> {
        // TODO: Implement push task details
        Ok("task_details_placeholder".to_string())
    }

    /// 获取兼容性矩阵
    pub async fn get_compatibility_matrix(&self) -> Result<String, AppError> {
        // TODO: Implement compatibility matrix
        Ok("matrix_placeholder".to_string())
    }

    /// 获取更新仪表板
    pub async fn get_update_dashboard(&self) -> Result<String, AppError> {
        // TODO: Implement update dashboard
        Ok("dashboard_placeholder".to_string())
    }
}

/// 版本统计
#[derive(Debug, Clone, serde::Serialize)]
pub struct VersionStatistics {
    pub total: i64,
    pub available: i64,
    pub deprecated: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_semantic_version() {
        let service = VersionService::new(
            VersionRepository::new(sqlx::SqlitePool::connect("").await.unwrap()),
            DeviceRepository::new(sqlx::SqlitePool::connect("").await.unwrap()),
            AuditLogRepository::new(sqlx::SqlitePool::connect("").await.unwrap()),
        );

        assert!(service.validate_semantic_version("1.0.0").is_ok());
        assert!(service.validate_semantic_version("1.2.3").is_ok());
        assert!(service.validate_semantic_version("10.20.30").is_ok());

        assert!(service.validate_semantic_version("1.0").is_err());
        assert!(service.validate_semantic_version("1.0.0.0").is_err());
        assert!(service.validate_semantic_version("1.a.0").is_err());
    }

    #[tokio::test]
    async fn test_compare_versions() {
        let service = VersionService::new(
            VersionRepository::new(sqlx::SqlitePool::connect("").await.unwrap()),
            DeviceRepository::new(sqlx::SqlitePool::connect("").await.unwrap()),
            AuditLogRepository::new(sqlx::SqlitePool::connect("").await.unwrap()),
        );

        assert!(service.is_newer_version("1.0.1", "1.0.0"));
        assert!(service.is_newer_version("1.1.0", "1.0.9"));
        assert!(service.is_newer_version("2.0.0", "1.9.9"));
        assert!(!service.is_newer_version("1.0.0", "1.0.0"));
        assert!(!service.is_newer_version("1.0.0", "1.0.1"));
    }
}
