use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    api::{middleware::extract_user_id, AppState},
    dto::{
        request::{ApproveDeviceRequest, RegisterDeviceRequest, RejectDeviceRequest},
        response::{DeviceListResponse, DeviceResponse, RegisterDeviceResponse, DeviceStatisticsResponse},
    },
    models::DeviceStatus,
    utils::error::AppError,
};

/// 设备列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListDevicesQuery {
    pub status: Option<DeviceStatus>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 设备注册处理器
///
/// POST /api/v1/devices/register
pub async fn register_device(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<RegisterDeviceRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = claims.sub;

    // 调用服务层
    let response = state
        .device_service
        .register_device(req, &operator_id)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// 设备列表处理器
///
/// GET /api/v1/devices
pub async fn list_devices(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListDevicesQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state
        .device_service
        .list_devices(
            query.status,
            query.search.as_deref(),
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取设备详情处理器
///
/// GET /api/v1/devices/:device_id
pub async fn get_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state.device_service.get_device(&device_id).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 审批设备处理器
///
/// POST /api/v1/devices/:device_id/approve
pub async fn approve_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<ApproveDeviceRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = claims.sub;

    // 调用服务层
    let response = state
        .device_service
        .approve_device(req)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 拒绝设备处理器
///
/// POST /api/v1/devices/:device_id/reject
pub async fn reject_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<RejectDeviceRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = claims.sub;

    // 调用服务层
    let response = state
        .device_service
        .reject_device(req)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 暂停设备处理器
///
/// POST /api/v1/devices/:device_id/suspend
pub async fn suspend_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
    request: axum::extract::Request,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = extract_user_id(&request)?;

    // 调用服务层
    state
        .device_service
        .suspend_device(&device_id, &operator_id, "Manual suspension")
        .await?;

    #[derive(Serialize)]
    struct SuspendResponse {
        message: String,
        device_id: String,
    }

    let response = SuspendResponse {
        message: "Device suspended successfully".to_string(),
        device_id,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// 恢复设备处理器
///
/// POST /api/v1/devices/:device_id/resume
pub async fn resume_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
    request: axum::extract::Request,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = extract_user_id(&request)?;

    // 调用服务层
    state
        .device_service
        .resume_device(&device_id, &operator_id)
        .await?;

    #[derive(Serialize)]
    struct ResumeResponse {
        message: String,
        device_id: String,
    }

    let response = ResumeResponse {
        message: "Device resumed successfully".to_string(),
        device_id,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// 吊销设备处理器
///
/// POST /api/v1/devices/:device_id/revoke
pub async fn revoke_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
    request: axum::extract::Request,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = extract_user_id(&request)?;

    // 调用服务层
    state
        .device_service
        .revoke_device(&device_id, &operator_id, "Manual revocation")
        .await?;

    #[derive(Serialize)]
    struct RevokeResponse {
        message: String,
        device_id: String,
    }

    let response = RevokeResponse {
        message: "Device revoked successfully".to_string(),
        device_id,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// 获取设备统计信息处理器
///
/// GET /api/v1/devices/statistics
pub async fn get_device_statistics(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let stats = state.device_service.get_device_statistics().await?;

    let response = DeviceStatisticsResponse {
        total: stats.total,
        active: stats.active,
        pending: stats.pending,
        suspended: stats.suspended,
        revoked: stats.revoked,
        average_security_score: stats.average_security_score,
    };

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices_query_defaults() {
        let query = ListDevicesQuery {
            status: None,
            search: None,
            sort_by: None,
            sort_order: None,
            page: None,
            page_size: None,
        };

        assert_eq!(query.page.unwrap_or(1), 1);
        assert_eq!(query.page_size.unwrap_or(20), 20);
    }
}
