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
    dto::{
        request::HealthCheckRequest,
        response::{HealthCheckResponse, HealthOverviewResponse},
    },
    utils::error::AppError,
};

/// 健康检查列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListHealthChecksQuery {
    pub device_id: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 提交健康检查处理器
///
/// POST /api/v1/health/submit
pub async fn submit_health_check(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<HealthCheckRequest>,
) -> Result<impl IntoResponse, AppError> {
    let device_id = req.device_id.clone();
    
    // 提取操作员ID
    let operator_id = claims.sub;

    // 调用服务层
    let response = state.health_check_service.submit_health_check(req, &operator_id).await?;

    // 发送通知（如果安全评分低于60）
    if response.security_score < 60 {
        let message = format!(
            "设备 {} 安全评分降至 {}，建议操作：{:?}",
            device_id, response.security_score, response.recommended_action
        );
        
        // 异步发送通知，不阻塞响应
        let notification_service = state.notification_service.clone();
        tokio::spawn(async move {
            notification_service
                .send_security_alert(device_id, response.security_score, message)
                .await;
        });
    }

    Ok((StatusCode::OK, Json(response)))
}

/// 列出健康检查记录处理器
///
/// GET /api/v1/health/checks
pub async fn list_health_checks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListHealthChecksQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state
        .health_check_service
        .list_health_checks(
            query.device_id.as_deref(),
            query.start_time.as_deref(),
            query.end_time.as_deref(),
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20),
        )
        .await?;

    let response: crate::dto::response::HealthCheckListResponse = response;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取健康概览处理器
///
/// GET /api/v1/health/:device_id/overview
pub async fn get_health_overview(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state
        .health_check_service
        .get_health_overview(&device_id)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 执行初始健康检查处理器
///
/// POST /api/v1/health/:device_id/initial-check
pub async fn perform_initial_check(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state
        .health_check_service
        .perform_initial_check(&device_id)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 系统健康检查处理器
///
/// GET /api/v1/health/check
pub async fn health_check(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    // 检查应用状态
    state
        .health_check()
        .await
        .map_err(|e| AppError::ServiceUnavailable)?;

    #[derive(serde::Serialize)]
    struct HealthCheckResponse {
        status: String,
        timestamp: String,
        services: ServicesStatus,
    }

    #[derive(serde::Serialize)]
    struct ServicesStatus {
        database: String,
        redis: String,
        hsm: String,
    }

    let response = HealthCheckResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        services: ServicesStatus {
            database: "ok".to_string(),
            redis: if state.redis_client.is_some() {
                "ok"
            } else {
                "not_configured"
            }
            .to_string(),
            hsm: if state.hsm_client.is_some() {
                "ok"
            } else {
                "not_configured"
            }
            .to_string(),
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// 获取健康统计信息处理器
///
/// GET /api/v1/health/statistics
pub async fn get_health_statistics(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // 获取所有设备的健康概览
    // 这里简化实现，实际应该有专门的统计方法
    #[derive(serde::Serialize)]
    struct HealthStatistics {
        total_checks: i64,
        average_score: f64,
        devices_with_low_score: i64,
        recent_threats: i64,
    }

    // TODO: 实现真实的统计逻辑
    let stats = HealthStatistics {
        total_checks: 0,
        average_score: 0.0,
        devices_with_low_score: 0,
        recent_threats: 0,
    };

    Ok((StatusCode::OK, Json(stats)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_health_checks_query_defaults() {
        let query = ListHealthChecksQuery {
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
