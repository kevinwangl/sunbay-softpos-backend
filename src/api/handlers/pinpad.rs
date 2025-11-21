use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    api::AppState,
    dto::request::{AttestPinpadRequest, EncryptPinRequest},
    utils::error::AppError,
};

/// PIN加密日志查询参数
#[derive(Debug, Deserialize)]
pub struct ListPinEncryptionLogsQuery {
    pub device_id: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// PINPad设备鉴证处理器
///
/// POST /api/v1/pinpad/attest
pub async fn attest_pinpad(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AttestPinpadRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 调用设备服务的PINPad鉴证方法
    let response = state.device_service.attest_pinpad_device(req).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// PIN加密处理器
///
/// POST /api/v1/pinpad/encrypt
pub async fn encrypt_pin(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<EncryptPinRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = claims.sub;

    // 调用密钥管理服务的PIN加密方法
    let response = state.key_management_service.encrypt_pin(req, &operator_id).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 列出PIN加密日志处理器
///
/// GET /api/v1/pinpad/logs
pub async fn list_pin_encryption_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListPinEncryptionLogsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 从审计日志中获取PIN加密相关的日志
    let response = state
        .audit_service
        .list_logs(
            Some("PIN_ENCRYPTION"),
            None,
            query.device_id.as_deref(),
            None,
            query.start_time.as_deref(),
            query.end_time.as_deref(),
            query.page_size.unwrap_or(20),
            (query.page.unwrap_or(1) - 1) * query.page_size.unwrap_or(20),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取设备PIN加密统计处理器
///
/// GET /api/v1/pinpad/device/:device_id/statistics
pub async fn get_device_pin_statistics(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 获取特定设备的PIN加密统计
    #[derive(serde::Serialize)]
    struct PinStatistics {
        device_id: String,
        total_encryptions: i64,
        successful_encryptions: i64,
        failed_encryptions: i64,
        last_encryption_time: Option<String>,
    }

    // TODO: 实现真实的统计逻辑
    let stats = PinStatistics {
        device_id,
        total_encryptions: 0,
        successful_encryptions: 0,
        failed_encryptions: 0,
        last_encryption_time: None,
    };

    Ok((StatusCode::OK, Json(stats)))
}

/// 验证PINPad设备状态处理器
///
/// GET /api/v1/pinpad/device/:device_id/status
pub async fn get_pinpad_device_status(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 获取设备信息并验证是否为PINPad模式
    let device = state.device_service.get_device(&device_id).await?;

    #[derive(serde::Serialize)]
    struct PinpadStatusResponse {
        device_id: String,
        is_pinpad_mode: bool,
        status: String,
        security_score: i32,
        key_status: String,
    }

    let response = PinpadStatusResponse {
        device_id: device.id.clone(),
        is_pinpad_mode: matches!(device.device_mode, crate::models::DeviceMode::PinPad),
        status: format!("{:?}", device.status),
        security_score: device.security_score,
        key_status: if device.ksn.is_some() {
            "injected"
        } else {
            "not_injected"
        }
        .to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_pin_encryption_logs_query_defaults() {
        let query = ListPinEncryptionLogsQuery {
            device_id: None,
            start_time: None,
            end_time: None,
            page: None,
            page_size: None,
        };

        assert_eq!(query.page.unwrap_or(1), 1);
        assert_eq!(query.page_size.unwrap_or(20), 20);
    }
}
