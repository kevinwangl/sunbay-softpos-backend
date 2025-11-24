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
        request::{AttestTransactionRequest, ProcessTransactionRequest},
        response::{AttestTransactionResponse, ProcessTransactionResponse},
    },
    models::TransactionStatus,
    utils::error::AppError,
};

/// 交易列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListTransactionsQuery {
    pub device_id: Option<String>,
    pub status: Option<TransactionStatus>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 交易鉴证处理器
///
/// POST /api/v1/transactions/attest
pub async fn attest_transaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<AttestTransactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = claims.sub;

    // 调用服务层
    let response = state.transaction_service.attest_transaction(req, &operator_id).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 交易处理处理器
///
/// POST /api/v1/transactions/process
pub async fn process_transaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<ProcessTransactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = claims.sub;

    // 调用服务层
    let response = state.transaction_service.process_transaction(req, &operator_id).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 列出交易记录处理器
///
/// GET /api/v1/transactions
pub async fn list_transactions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListTransactionsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response_data = state
        .transaction_service
        .list_transactions(
            query.device_id.as_deref(),
            query.status,
            None, // transaction_type
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20),
        )
        .await?;

    let wrapped_response = serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": response_data
    });

    Ok((StatusCode::OK, Json(wrapped_response)))
}

/// 获取交易详情处理器
///
/// GET /api/v1/transactions/:transaction_id
pub async fn get_transaction(
    State(state): State<Arc<AppState>>,
    Path(transaction_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层
    let response = state
        .transaction_service
        .get_transaction(&transaction_id)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取设备交易历史处理器
///
/// GET /api/v1/transactions/device/:device_id/history
pub async fn get_device_transaction_history(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
    Query(query): Query<ListTransactionsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 调用服务层，只获取特定设备的交易
    let response = state
        .transaction_service
        .list_transactions(
            Some(&device_id),
            query.status,
            None, // transaction_type
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 获取交易统计信息处理器
///
/// GET /api/v1/transactions/statistics
pub async fn get_transaction_statistics(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // 获取交易统计
    // 简化实现，实际应该有专门的统计方法
    #[derive(serde::Serialize)]
    struct TransactionStatistics {
        total_transactions: i64,
        successful_transactions: i64,
        failed_transactions: i64,
        total_amount: i64,
        success_rate: f64,
    }

    // TODO: 实现真实的统计逻辑
    let stats = TransactionStatistics {
        total_transactions: 0,
        successful_transactions: 0,
        failed_transactions: 0,
        total_amount: 0,
        success_rate: 0.0,
    };

    Ok((StatusCode::OK, Json(stats)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_transactions_query_defaults() {
        let query = ListTransactionsQuery {
            device_id: None,
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
