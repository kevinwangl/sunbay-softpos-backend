use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::{api::AppState, security::jwt::Claims, utils::error::AppError};

/// JWT认证中间件
///
/// 从请求头中提取JWT token，验证并将Claims注入到请求扩展中
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 从Authorization头中提取token
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    // 检查Bearer前缀
    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized(
            "Invalid authorization header format".to_string(),
        ));
    }

    // 提取token
    let token = auth_header.trim_start_matches("Bearer ").trim();

    // 验证token
    let claims = state
        .jwt_service
        .verify_token(token)
        .map_err(|e| AppError::Unauthorized(format!("Token verification failed: {}", e)))?;

    // 将Claims注入到请求扩展中，供后续处理器使用
    request.extensions_mut().insert(claims);

    // 继续处理请求
    Ok(next.run(request).await)
}

/// 可选的JWT认证中间件
///
/// 如果存在token则验证，不存在则继续处理
pub async fn optional_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    // 尝试从Authorization头中提取token
    if let Some(auth_header) = request.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str.trim_start_matches("Bearer ").trim();

                // 尝试验证token
                if let Ok(claims) = state.jwt_service.verify_token(token) {
                    request.extensions_mut().insert(claims);
                }
            }
        }
    }

    // 继续处理请求（无论是否有有效token）
    next.run(request).await
}

/// 从请求扩展中提取Claims
pub fn extract_claims(request: &Request) -> Result<Claims, AppError> {
    request
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("No authentication claims found".to_string()))
}

/// 从请求扩展中提取用户ID
pub fn extract_user_id(request: &Request) -> Result<String, AppError> {
    let claims = extract_claims(request)?;
    Ok(claims.sub)
}

/// 从请求扩展中提取用户名
pub fn extract_username(request: &Request) -> Result<String, AppError> {
    let claims = extract_claims(request)?;
    Ok(claims.username)
}

/// 从请求扩展中提取用户角色
pub fn extract_role(request: &Request) -> Result<String, AppError> {
    let claims = extract_claims(request)?;
    Ok(claims.role)
}

/// 检查用户是否具有指定角色
pub fn has_role(request: &Request, required_role: &str) -> Result<bool, AppError> {
    let role = extract_role(request)?;
    Ok(role == required_role)
}

/// 角色检查中间件
///
/// 检查用户是否具有指定角色
pub async fn require_role(
    required_role: String,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, AppError>> + Send>>
{
    move |request: Request, next: Next| {
        let required_role = required_role.clone();
        Box::pin(async move {
            let claims = extract_claims(&request)?;

            if claims.role != required_role {
                return Err(AppError::Unauthorized(format!(
                    "Required role: {}",
                    required_role
                )));
            }

            Ok(next.run(request).await)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request as HttpRequest, StatusCode},
    };

    #[tokio::test]
    async fn test_extract_claims() {
        let mut request = HttpRequest::new(Body::empty());

        let claims = Claims {
            sub: "user123".to_string(),
            username: "testuser".to_string(),
            role: "admin".to_string(),
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
        };

        request.extensions_mut().insert(claims.clone());

        let extracted = extract_claims(&request).unwrap();
        assert_eq!(extracted.sub, "user123");
        assert_eq!(extracted.username, "testuser");
        assert_eq!(extracted.role, "admin");
    }

    #[tokio::test]
    async fn test_extract_user_id() {
        let mut request = HttpRequest::new(Body::empty());

        let claims = Claims {
            sub: "user123".to_string(),
            username: "testuser".to_string(),
            role: "admin".to_string(),
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
        };

        request.extensions_mut().insert(claims);

        let user_id = extract_user_id(&request).unwrap();
        assert_eq!(user_id, "user123");
    }

    #[tokio::test]
    async fn test_has_role() {
        let mut request = HttpRequest::new(Body::empty());

        let claims = Claims {
            sub: "user123".to_string(),
            username: "testuser".to_string(),
            role: "admin".to_string(),
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
        };

        request.extensions_mut().insert(claims);

        assert!(has_role(&request, "admin").unwrap());
        assert!(!has_role(&request, "user").unwrap());
    }
}
