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
    models::{ThreatSeverity, ThreatStatus, ThreatType},
    utils::error::AppError,
};

/// 威胁列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListThreatsQuery {
    pub device_id: Option<String>,
    pub threat_type: Option<ThreatType>,
    pub severity: Option<ThreatSeverity>,
    pub status: Option<ThreatStatus>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 解决威胁请求
#[derive(Debug, Deserialize)]
pub struct ResolveThreatRequest {
    pub resolution_notes: Option<String>,
}

/// 列出威胁事件处理器
///
/// GET /api/v1/threats
pub async fn list_threats(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListThreatsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state
        .threat_detection_service
        .list_threats(
            query.device_id.as_deref(),
            query.status,
            query.severity,
            query.threat_type,
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取威胁详情处理器
///
/// GET /api/v1/threats/:threat_id
pub async fn get_threat(
    State(state): State<Arc<AppState>>,
    Path(threat_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层获取威胁详情
    // 注意：这里需要从list_threats中筛选单个威胁
    // 简化实现，实际应该有专门的get_threat方法
    let threats = state
        .threat_detection_service
        .list_threats(None, None, None, None, 1, 1000)
        .await?;

    let threat = threats
        .threats
        .into_iter()
        .find(|t| t.id == threat_id)
        .ok_or_else(|| AppError::ThreatNotFound)?;

    Ok((StatusCode::OK, Json(threat)))
}

/// 解决威胁处理器
///
/// POST /api/v1/threats/:threat_id/resolve
pub async fn resolve_threat(
    State(state): State<Arc<AppState>>,
    Path(threat_id): Path<String>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<ResolveThreatRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = claims.sub;

    // 调用服务层
    state
        .threat_detection_service
        .resolve_threat(&threat_id, &operator_id, req.resolution_notes)
        .await?;

    #[derive(serde::Serialize)]
    struct ResolveResponse {
        message: String,
        threat_id: String,
    }

    let response = ResolveResponse {
        message: "Threat resolved successfully".to_string(),
        threat_id,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// 获取威胁统计信息处理器
///
/// GET /api/v1/threats/statistics
pub async fn get_threat_statistics(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let stats = state
        .threat_detection_service
        .get_threat_statistics()
        .await?;

    Ok((StatusCode::OK, Json(stats)))
}

/// 获取设备威胁历史处理器
///
/// GET /api/v1/threats/device/:device_id/history
pub async fn get_device_threat_history(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
    Query(query): Query<ListThreatsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层，只获取特定设备的威胁
    let response = state
        .threat_detection_service
        .list_threats(
            Some(&device_id),
            query.status,
            query.severity,
            query.threat_type,
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_threats_query_defaults() {
        let query = ListThreatsQuery {
            device_id: None,
            threat_type: None,
            severity: None,
            status: None,
            start_time: None,
            end_time: None,
            page: None,
            page_size: None,
        };

        assert_eq!(query.page.unwrap_or(1), 1);
        assert_eq!(query.page_size.unwrap_or(20), 20);
    }
}
