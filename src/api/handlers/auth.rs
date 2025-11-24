use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    api::AppState,
    dto::{request::LoginRequest, response::LoginResponse},
    utils::error::AppError,
    models::OperationResult,
};

/// 刷新Token请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// 刷新Token响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// 登出响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutResponse {
    pub message: String,
}

/// 登录处理器
///
/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 验证请求
    request
        .validate()
        .map_err(|e| AppError::Validation(e))?;

    // TODO: 实际应用中应该从数据库验证用户凭证
    // 这里使用硬编码的测试用户
    let (user_id, username, role) = if request.username == "admin" && request.password == "admin123" {
        ("admin_001".to_string(), "admin".to_string(), "admin".to_string())
    } else if request.username == "operator" && request.password == "operator123" {
        ("operator_001".to_string(), "operator".to_string(), "operator".to_string())
    } else {
        return Err(AppError::InvalidCredentials);
    };

    // 生成access token
    let access_token = state
        .jwt_service
        .generate_token(&user_id, &username, &role)?;

    // 生成refresh token
    let refresh_token = state
        .jwt_service
        .generate_refresh_token(&user_id, &username, &role)?;

    // 记录审计日志
    state
        .audit_service
        .log_operation(
            "USER_LOGIN".to_string(),
            user_id.clone(),
            OperationResult::Success,
            None,
            Some(serde_json::json!({
                "username": username,
                "role": role,
            }).to_string()),
        )
        .await
        .ok(); // 忽略审计日志错误

    // 构建响应
    let response_data = LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt.expiration_hours * 3600,
        user: crate::dto::response::UserInfo {
            user_id,
            username,
            role,
        },
    };

    let wrapped_response = serde_json::json!({
        "code": 200,
        "message": "Login successful",
        "data": response_data
    });

    Ok((StatusCode::OK, Json(wrapped_response)))
}

/// 刷新Token处理器
///
/// POST /api/v1/auth/refresh
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 验证并刷新token
    let (new_access_token, new_refresh_token) = state
        .jwt_service
        .refresh_token(&request.refresh_token)?;

    // 从旧token中提取用户信息用于审计
    if let Ok(claims) = state.jwt_service.verify_token(&request.refresh_token) {
        state
            .audit_service
            .log_operation(
                "TOKEN_REFRESH".to_string(),
                claims.sub.clone(),
                OperationResult::Success,
                None,
                Some(serde_json::json!({
                    "username": claims.username,
                }).to_string()),
            )
            .await
            .ok();
    }

    // 构建响应
    let response_data = RefreshTokenResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt.expiration_hours * 3600,
    };

    let wrapped_response = serde_json::json!({
        "code": 200,
        "message": "Token refreshed successfully",
        "data": response_data
    });

    Ok((StatusCode::OK, Json(wrapped_response)))
}

/// 登出处理器
///
/// POST /api/v1/auth/logout
pub async fn logout(
    State(state): State<Arc<AppState>>,
    request: axum::extract::Request,
) -> Result<impl IntoResponse, AppError> {
    // 从请求扩展中提取用户信息
    let claims = crate::api::middleware::extract_claims(&request)?;

    // 记录审计日志
    state
        .audit_service
        .log_operation(
            "USER_LOGOUT".to_string(),
            claims.sub.clone(),
            OperationResult::Success,
            None,
            Some(serde_json::json!({
                "username": claims.username,
            }).to_string()),
        )
        .await
        .ok();

    // TODO: 在实际应用中，应该将token加入黑名单
    // 可以使用Redis存储已登出的token，直到它们过期

    let response = LogoutResponse {
        message: "Logged out successfully".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// 获取当前用户信息处理器
///
/// GET /api/v1/auth/me
pub async fn get_current_user(
    request: axum::extract::Request,
) -> Result<impl IntoResponse, AppError> {
    // 从请求扩展中提取用户信息
    let claims = crate::api::middleware::extract_claims(&request)?;

    let user_info = crate::dto::response::UserInfo {
        user_id: claims.sub,
        username: claims.username,
        role: claims.role,
    };

    Ok((StatusCode::OK, Json(user_info)))
}

/// 验证Token处理器
///
/// POST /api/v1/auth/verify
pub async fn verify_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let token = request
        .get("token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing token field".to_string()))?;

    // 验证token
    let claims = state.jwt_service.verify_token(token)?;

    #[derive(Serialize)]
    struct VerifyResponse {
        valid: bool,
        user_id: String,
        username: String,
        role: String,
        exp: i64,
    }

    let response = VerifyResponse {
        valid: true,
        user_id: claims.sub,
        username: claims.username,
        role: claims.role,
        exp: claims.exp,
    };

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_validation() {
        let valid_request = LoginRequest {
            username: "admin".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = LoginRequest {
            username: "".to_string(),
            password: "password123".to_string(),
        };
        assert!(invalid_request.validate().is_err());

        let invalid_request = LoginRequest {
            username: "admin".to_string(),
            password: "".to_string(),
        };
        assert!(invalid_request.validate().is_err());
    }
}
