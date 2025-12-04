use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};

use crate::{
    api::{middleware::extract_user_id, AppState},
    dto::{
        request::{EncryptPinRequest, InjectKeyRequest, UpdateKeyRequest},
        response::{InjectKeyResponse, KeyStatusResponse, UpdateKeyResponse},
    },
    utils::error::AppError,
};

/// 密钥注入处理器
///
/// POST /api/v1/keys/inject
pub async fn inject_key(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<InjectKeyRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = claims.sub;

    // 调用服务层
    let response = state.key_management_service.inject_key(req, &operator_id).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 公开密钥注入处理器 (Demo专用)
///
/// POST /api/v1/public/keys/inject
pub async fn inject_key_public(
    State(state): State<Arc<AppState>>,
    Json(req): Json<InjectKeyRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 使用默认的 demo 操作员 ID
    let operator_id = "demo-operator".to_string();

    // 调用服务层
    let response = state.key_management_service.inject_key(req, &operator_id).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取密钥状态处理器
///
/// GET /api/v1/keys/:device_id/status
pub async fn get_key_status(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state.key_management_service.get_key_status(&device_id).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 更新密钥处理器
///
/// POST /api/v1/keys/:device_id/update
pub async fn update_key(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<UpdateKeyRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = claims.sub;

    // 调用服务层
    let response = state.key_management_service.update_key(req, &operator_id).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 加密PIN处理器
///
/// POST /api/v1/keys/encrypt-pin
pub async fn encrypt_pin(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<EncryptPinRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = claims.sub;

    // 调用服务层
    let response = state.key_management_service.encrypt_pin(req, &operator_id).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 检查密钥是否需要更新处理器
///
/// GET /api/v1/keys/:device_id/check-update
pub async fn check_key_update_needed(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let needs_update = state.key_management_service.check_key_update_needed(&device_id).await?;

    #[derive(serde::Serialize)]
    struct CheckUpdateResponse {
        device_id: String,
        needs_update: bool,
    }

    let response = CheckUpdateResponse {
        device_id,
        needs_update,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// 获取需要更新密钥的设备列表处理器
///
/// GET /api/v1/keys/devices-needing-update
pub async fn get_devices_needing_key_update(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let device_ids = state.key_management_service.get_devices_needing_key_update().await?;

    #[derive(serde::Serialize)]
    struct DevicesNeedingUpdateResponse {
        device_ids: Vec<String>,
        count: usize,
    }

    let response = DevicesNeedingUpdateResponse {
        count: device_ids.len(),
        device_ids,
    };

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_management_handlers_exist() {
        // 简单的编译时测试，确保所有处理器函数存在
        let _ = inject_key;
        let _ = get_key_status;
        let _ = update_key;
        let _ = encrypt_pin;
        let _ = check_key_update_needed;
        let _ = get_devices_needing_key_update;
    }
}
