use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{api::AppState, utils::error::AppError};

/// 审计日志列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListAuditLogsQuery {
    pub device_id: Option<String>,
    pub operation_type: Option<String>,
    pub operator: Option<String>,
    pub result: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 列出审计日志处理器
///
/// GET /api/v1/audit/logs
pub async fn list_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListAuditLogsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state
        .audit_service
        .list_logs(
            query.operation_type.as_deref(),
            None, // operator
            query.device_id.as_deref(),
            None, // result
            query.start_time.as_deref(),
            query.end_time.as_deref(),
            query.page_size.unwrap_or(20),
            (query.page.unwrap_or(1) - 1) * query.page_size.unwrap_or(20),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取审计日志详情处理器
///
/// GET /api/v1/audit/logs/:log_id
pub async fn get_log(
    State(state): State<Arc<AppState>>,
    Path(log_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state.audit_service.get_log(&log_id).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取设备审计日志处理器
///
/// GET /api/v1/audit/device/:device_id/logs
pub async fn get_device_logs(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
    Query(query): Query<ListAuditLogsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层，只获取特定设备的日志
    let response = state
        .audit_service
        .list_logs(
            query.operation_type.as_deref(),
            None, // operator
            Some(&device_id),
            None, // result
            query.start_time.as_deref(),
            query.end_time.as_deref(),
            query.page_size.unwrap_or(20),
            (query.page.unwrap_or(1) - 1) * query.page_size.unwrap_or(20),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取操作员审计日志处理器
///
/// GET /api/v1/audit/operator/:operator_id/logs
pub async fn get_operator_logs(
    State(state): State<Arc<AppState>>,
    Path(operator_id): Path<String>,
    Query(query): Query<ListAuditLogsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 获取特定操作员的日志
    // 简化实现，实际应该有专门的方法
    let response = state
        .audit_service
        .list_logs(
            query.operation_type.as_deref(),
            Some(&operator_id),
            query.device_id.as_deref(),
            None, // result
            query.start_time.as_deref(),
            query.end_time.as_deref(),
            query.page_size.unwrap_or(20),
            (query.page.unwrap_or(1) - 1) * query.page_size.unwrap_or(20),
        )
        .await?;

    // TODO: 过滤operator_id
    Ok((StatusCode::OK, Json(response)))
}

/// 获取审计日志统计处理器
///
/// GET /api/v1/audit/statistics
pub async fn get_audit_statistics(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // 获取审计日志统计
    #[derive(serde::Serialize)]
    struct AuditStatistics {
        total_logs: i64,
        operations_by_type: std::collections::HashMap<String, i64>,
        operations_by_result: std::collections::HashMap<String, i64>,
        recent_operations: i64,
    }

    // TODO: 实现真实的统计逻辑
    let stats = AuditStatistics {
        total_logs: 0,
        operations_by_type: std::collections::HashMap::new(),
        operations_by_result: std::collections::HashMap::new(),
        recent_operations: 0,
    };

    Ok((StatusCode::OK, Json(stats)))
}

/// 导出审计日志处理器
///
/// GET /api/v1/audit/export
pub async fn export_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListAuditLogsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 获取日志
    let response = state
        .audit_service
        .list_logs(
            query.operation_type.as_deref(),
            None,
            query.device_id.as_deref(),
            None,
            query.start_time.as_deref(),
            query.end_time.as_deref(),
            10000, // limit
            0, // offset
        )
        .await?;

    // TODO: 转换为CSV格式
    // 这里简化实现，返回JSON
    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_audit_logs_query_defaults() {
        let query = ListAuditLogsQuery {
            device_id: None,
            operation_type: None,
            operator: None,
            result: None,
            start_time: None,
            end_time: None,
            page: None,
            page_size: None,
        };

        assert_eq!(query.page.unwrap_or(1), 1);
        assert_eq!(query.page_size.unwrap_or(20), 20);
    }
}
