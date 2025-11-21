use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    api::{middleware::extract_user_id, AppState},
    dto::{
        request::{CreateVersionRequest, UpdateVersionRequest, CreatePushTaskRequest},
        response::VersionResponse,
    },
    models::{UpdateType, VersionStatus},
    utils::error::AppError,
};

/// 版本列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListVersionsQuery {
    pub status: Option<VersionStatus>,
    pub update_type: Option<UpdateType>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 推送任务列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListPushTasksQuery {
    pub version_id: Option<String>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 创建版本处理器
///
/// POST /api/v1/versions
pub async fn create_version(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<CreateVersionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = claims.sub;

    // 调用服务层
    let response = state.version_service.create_version(req, &operator_id).await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// 列出版本处理器
///
/// GET /api/v1/versions
pub async fn list_versions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListVersionsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state
        .version_service
        .list_versions(
            query.status,
            query.update_type,
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取版本详情处理器
///
/// GET /api/v1/versions/:version_id
pub async fn get_version(
    State(state): State<Arc<AppState>>,
    Path(version_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state.version_service.get_version(&version_id).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 更新版本处理器
///
/// PUT /api/v1/versions/:version_id
pub async fn update_version(
    State(state): State<Arc<AppState>>,
    Path(version_id): Path<String>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<UpdateVersionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = claims.sub;

    // 调用服务层
    let response = state
        .version_service
        .update_version(&version_id, req, &operator_id)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取版本统计信息处理器
///
/// GET /api/v1/versions/statistics
pub async fn get_version_statistics(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let stats = state.version_service.get_version_statistics().await?;

    Ok((StatusCode::OK, Json(stats)))
}

/// 获取兼容性矩阵处理器
///
/// GET /api/v1/versions/compatibility
pub async fn get_compatibility_matrix(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let matrix = state.version_service.get_compatibility_matrix().await?;

    Ok((StatusCode::OK, Json(matrix)))
}

/// 创建推送任务处理器
///
/// POST /api/v1/versions/push
pub async fn create_push_task(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<CreatePushTaskRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = claims.sub;

    // 调用服务层
    let response = state
        .version_service
        .create_push_task(req, &operator_id)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// 列出推送任务处理器
///
/// GET /api/v1/versions/push
pub async fn list_push_tasks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListPushTasksQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state
        .version_service
        .list_push_tasks(
            query.version_id.as_deref(),
            query.status.as_deref(),
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取推送任务详情处理器
///
/// GET /api/v1/versions/push/:task_id
pub async fn get_push_task(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state.version_service.get_push_task(&task_id).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取设备可用版本处理器
///
/// GET /api/v1/versions/available/:device_id
pub async fn get_available_version(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state
        .version_service
        .get_available_version(&device_id)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取过期设备列表处理器
///
/// GET /api/v1/versions/outdated-devices
pub async fn get_outdated_devices(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state.version_service.get_outdated_devices().await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取更新仪表板处理器
///
/// GET /api/v1/versions/update-dashboard
pub async fn get_update_dashboard(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state.version_service.get_update_dashboard().await?;

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_versions_query_defaults() {
        let query = ListVersionsQuery {
            status: None,
            update_type: None,
            page: None,
            page_size: None,
        };

        assert_eq!(query.page.unwrap_or(1), 1);
        assert_eq!(query.page_size.unwrap_or(20), 20);
    }

    #[test]
    fn test_list_push_tasks_query_defaults() {
        let query = ListPushTasksQuery {
            version_id: None,
            status: None,
            page: None,
            page_size: None,
        };

        assert_eq!(query.page.unwrap_or(1), 1);
        assert_eq!(query.page_size.unwrap_or(20), 20);
    }
}
