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

/// 请求交易令牌处理器
///
/// POST /api/v1/transactions/request-token
pub async fn request_transaction_token(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<RequestTransactionTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 验证设备存在
    let device = state
        .device_service
        .get_device(&req.device_id)
        .await?;

    // 获取最新的健康检查记录
    let health_checks = state
        .health_check_service
        .list_health_checks(Some(&req.device_id), None, None, 1, 1)
        .await?;

    if health_checks.checks.is_empty() {
        return Err(AppError::BadRequest(
            "No health check found for device. Please perform health check first.".to_string()
        ));
    }

    let latest_check = &health_checks.checks[0];

    // 检查健康检查是否过期（5分钟内有效）
    let check_time = chrono::DateTime::parse_from_rfc3339(&latest_check.checked_at)
        .map_err(|e| AppError::InternalWithMessage(format!("Invalid timestamp: {}", e)))?;
    let now = chrono::Utc::now();
    let age = now.signed_duration_since(check_time.with_timezone(&chrono::Utc));

    if age.num_seconds() > 300 {
        return Err(AppError::BadRequest(
            "Health check expired. Please perform a new health check.".to_string()
        ));
    }

    // 构建HealthCheck对象
    let health_check = crate::models::HealthCheck {
        id: latest_check.check_id.clone(),
        device_id: req.device_id.clone(),
        security_score: latest_check.security_score,
        root_status: false,
        bootloader_status: false,
        system_integrity: false,
        app_integrity: false,
        tee_status: false,
        recommended_action: crate::models::RecommendedAction::None,
        details: None,
        created_at: latest_check.checked_at.clone(),
    };

    // 生成交易令牌
    let token = state
        .transaction_token_service
        .generate_token(&req.device_id, &health_check)
        .await?;

    tracing::info!(
        "Transaction token requested by {} for device {}",
        claims.username,
        req.device_id
    );

    Ok((StatusCode::OK, Json(token)))
}

/// 验证交易令牌处理器
///
/// POST /api/v1/transactions/verify-token
pub async fn verify_transaction_token(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerifyTransactionTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 验证令牌
    let claims = state
        .transaction_token_service
        .verify_token(&req.transaction_token, &req.device_id)
        .await?;

    #[derive(serde::Serialize)]
    struct VerifyTokenResponse {
        valid: bool,
        device_id: String,
        security_score: i32,
        max_amount: i64,
        expires_at: String,
    }

    let response = VerifyTokenResponse {
        valid: true,
        device_id: claims.sub,
        security_score: claims.security_score,
        max_amount: claims.max_amount,
        expires_at: chrono::DateTime::from_timestamp(claims.exp, 0)
            .unwrap()
            .to_rfc3339(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// 交易鉴证处理器（需要认证，管理端使用）
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

/// 交易鉴证处理器（公开，设备端使用）
///
/// POST /api/v1/transactions/attest
pub async fn attest_transaction_public(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AttestTransactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 设备端调用，使用设备ID作为操作员ID
    let operator_id = format!("device:{}", req.device_id);

    // 调用服务层
    let response = state.transaction_service.attest_transaction(req, &operator_id).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// 请求交易令牌请求
#[derive(Debug, Deserialize)]
pub struct RequestTransactionTokenRequest {
    pub device_id: String,
}

/// 验证交易令牌请求
#[derive(Debug, Deserialize)]
pub struct VerifyTransactionTokenRequest {
    pub transaction_token: String,
    pub device_id: String,
}

/// 交易处理处理器（需要认证，管理端使用）
///
/// POST /api/v1/transactions/process
pub async fn process_transaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::security::jwt::Claims>,
    Json(req): Json<ProcessTransactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 提取操作员ID
    let operator_id = claims.sub;

    // 1. 验证交易令牌
    let token_claims = state
        .transaction_token_service
        .verify_token(&req.transaction_token, &req.device_id)
        .await?;

    tracing::info!(
        "Transaction token verified for device {}, jti: {}",
        req.device_id,
        token_claims.jti
    );

    // 2. 验证交易金额不超过令牌允许的最大金额
    if req.amount > token_claims.max_amount {
        return Err(AppError::BadRequest(format!(
            "Transaction amount {} exceeds maximum allowed amount {} for security score {}",
            req.amount, token_claims.max_amount, token_claims.security_score
        )));
    }

    // 3. 处理交易
    let response = state.transaction_service.process_transaction(req, &operator_id).await?;

    // 4. 标记令牌已使用
    if let Err(e) = state
        .transaction_token_service
        .mark_token_used(&token_claims, &response.transaction_id)
        .await
    {
        tracing::warn!(
            "Failed to mark transaction token as used: {}. Transaction {} completed successfully.",
            e,
            response.transaction_id
        );
        // 不阻塞交易响应，只记录警告
    }

    Ok((StatusCode::OK, Json(response)))
}

/// 交易处理处理器（公开，设备端使用）
///
/// POST /api/v1/transactions/process
pub async fn process_transaction_public(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ProcessTransactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 设备端调用，使用设备ID作为操作员ID
    let operator_id = format!("device:{}", req.device_id);

    // 1. 验证交易令牌
    let token_claims = state
        .transaction_token_service
        .verify_token(&req.transaction_token, &req.device_id)
        .await?;

    tracing::info!(
        "Transaction token verified for device {}, jti: {}",
        req.device_id,
        token_claims.jti
    );

    // 2. 验证交易金额不超过令牌允许的最大金额
    if req.amount > token_claims.max_amount {
        return Err(AppError::BadRequest(format!(
            "Transaction amount {} exceeds maximum allowed amount {} for security score {}",
            req.amount, token_claims.max_amount, token_claims.security_score
        )));
    }

    // 3. 处理交易
    let response = state.transaction_service.process_transaction(req, &operator_id).await?;

    // 4. 标记令牌已使用
    if let Err(e) = state
        .transaction_token_service
        .mark_token_used(&token_claims, &response.transaction_id)
        .await
    {
        tracing::warn!(
            "Failed to mark transaction token as used: {}. Transaction {} completed successfully.",
            e,
            response.transaction_id
        );
        // 不阻塞交易响应，只记录警告
    }

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
            query.page_size.unwrap_or(20), // limit
            (query.page.unwrap_or(1) - 1) * query.page_size.unwrap_or(20), // offset
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
            query.page_size.unwrap_or(20), // limit
            (query.page.unwrap_or(1) - 1) * query.page_size.unwrap_or(20), // offset
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
