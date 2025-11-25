use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{net::SocketAddr, time::Instant};
use tracing::{info, warn};

/// 请求日志中间件
///
/// 记录每个HTTP请求的详细信息
pub async fn logging_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();

    // 提取用户信息（如果已认证）
    let user_id = request
        .extensions()
        .get::<crate::security::jwt::Claims>()
        .map(|claims| claims.sub.clone());

    // 记录请求开始
    let start = Instant::now();

    info!(
        method = %method,
        uri = %uri,
        version = ?version,
        client_ip = %addr.ip(),
        user_id = ?user_id,
        "Incoming request"
    );

    // 处理请求
    let response = next.run(request).await;

    // 计算请求耗时
    let duration = start.elapsed();
    let status = response.status();

    // 根据状态码选择日志级别
    if status.is_server_error() {
        warn!(
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            client_ip = %addr.ip(),
            user_id = ?user_id,
            "Request completed with server error"
        );
    } else if status.is_client_error() {
        warn!(
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            client_ip = %addr.ip(),
            user_id = ?user_id,
            "Request completed with client error"
        );
    } else {
        info!(
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            client_ip = %addr.ip(),
            user_id = ?user_id,
            "Request completed successfully"
        );
    }

    response
}

/// 结构化日志中间件
///
/// 以JSON格式记录请求和响应信息
pub async fn structured_logging_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();
    let headers = request.headers().clone();

    // 提取用户信息
    let user_id = request
        .extensions()
        .get::<crate::security::jwt::Claims>()
        .map(|claims| claims.sub.clone());

    let start = Instant::now();

    // 处理请求
    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    // 构建结构化日志
    let log_entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "method": method.to_string(),
        "uri": uri.to_string(),
        "version": format!("{:?}", version),
        "status": status.as_u16(),
        "duration_ms": duration.as_millis(),
        "client_ip": addr.ip().to_string(),
        "user_id": user_id,
        "user_agent": headers.get("user-agent").and_then(|v| v.to_str().ok()),
        "referer": headers.get("referer").and_then(|v| v.to_str().ok()),
    });

    // 根据状态码选择日志级别
    if status.is_server_error() || status.is_client_error() {
        warn!(log = %log_entry, "HTTP request completed");
    } else {
        info!(log = %log_entry, "HTTP request completed");
    }

    response
}

/// 错误日志中间件
///
/// 专门记录错误响应的详细信息
pub async fn error_logging_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    let user_id = request
        .extensions()
        .get::<crate::security::jwt::Claims>()
        .map(|claims| claims.sub.clone());

    let response = next.run(request).await;
    let status = response.status();

    // 只记录错误响应
    if status.is_client_error() || status.is_server_error() {
        warn!(
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            client_ip = %addr.ip(),
            user_id = ?user_id,
            "Error response"
        );
    }

    response
}

/// 慢请求日志中间件
///
/// 记录超过阈值的慢请求
pub async fn slow_request_logging_middleware(
    threshold_ms: u64,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
{
    move |request: Request, next: Next| {
        Box::pin(async move {
            let method = request.method().clone();
            let uri = request.uri().clone();

            let start = Instant::now();
            let response = next.run(request).await;
            let duration = start.elapsed();

            if duration.as_millis() > threshold_ms as u128 {
                warn!(
                    method = %method,
                    uri = %uri,
                    duration_ms = duration.as_millis(),
                    threshold_ms = threshold_ms,
                    "Slow request detected"
                );
            }

            response
        })
    }
}

/// 请求ID中间件
///
/// 为每个请求生成唯一ID并注入到响应头中
pub async fn request_id_middleware(request: Request, next: Next) -> Response {
    // 生成请求ID
    let request_id = uuid::Uuid::new_v4().to_string();

    // 将请求ID添加到tracing span
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
    );

    let _enter = span.enter();

    // 处理请求
    let mut response = next.run(request).await;

    // 将请求ID添加到响应头
    response.headers_mut().insert(
        "X-Request-ID",
        request_id.parse().unwrap(),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request as HttpRequest, StatusCode},
    };

    #[tokio::test]
    async fn test_request_id_middleware() {
        use axum::{
            routing::get,
            Router,
        };
        use tower::ServiceExt; // for `oneshot`

        // 定义一个简单的处理函数
        async fn handler() -> impl IntoResponse {
            (StatusCode::OK, "test")
        }

        // 构建应用并添加中间件
        let app = Router::new()
            .route("/", get(handler))
            .layer(axum::middleware::from_fn(request_id_middleware));

        // 发送请求
        let request = HttpRequest::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // 验证响应状态码
        assert_eq!(response.status(), StatusCode::OK);

        // 验证响应头中包含X-Request-ID
        assert!(response.headers().contains_key("X-Request-ID"));
    }
}
