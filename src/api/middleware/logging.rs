use axum::{
    extract::{ConnectInfo, Request},
    middleware::Next,
    response::Response,
};
use std::{net::SocketAddr, time::Instant};
use tracing::{info, warn};

/// 请求日志中间件
///
/// 记录每个HTTP请求的详细信息，分离请求和响应日志
pub async fn logging_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_id = uuid::Uuid::new_v4();

    // 提取用户信息（如果已认证）
    let user_id = request
        .extensions()
        .get::<crate::security::jwt::Claims>()
        .map(|claims| claims.sub.clone());

    // 提取MatchedPath（路由模板）
    let matched_path = request
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|path| path.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string());

    let user_display = user_id.as_ref().map(|id| id.as_str()).unwrap_or("anonymous");

    // 记录请求开始
    let start = Instant::now();
    let start_time = chrono::Local::now();

    // 捕获请求体
    let (parts, body) = request.into_parts();
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(err) => {
            warn!("Failed to read request body: {}", err);
            return Response::builder()
                .status(400)
                .body(axum::body::Body::from("Failed to read request body"))
                .unwrap();
        },
    };

    // 尝试将请求体解析为 JSON 字符串以便打印
    let body_str = if !bytes.is_empty() {
        match std::str::from_utf8(&bytes) {
            Ok(s) => {
                // 尝试格式化 JSON
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(s) {
                    serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| s.to_string())
                } else {
                    s.to_string()
                }
            },
            Err(_) => format!("<binary data, {} bytes>", bytes.len()),
        }
    } else {
        String::from("<empty>")
    };

    // 构建Header
    let header = format!(
        "#################{} {}###########",
        start_time.format("%Y-%m-%dT%H:%M:%S%.6f"),
        uri.path()
    );

    let request_log = format!(
        "\n{}\n\
         ┌─ 📥 INCOMING REQUEST\n\
         │  Method: {} {}\n\
         │  Handler: {}\n\
         │  Client: {} | User: {}\n\
         │  Request ID: {}\n\
         │  Body:\n{}\n\
         └─ Processing...",
        header,
        method,
        uri,
        matched_path,
        addr.ip(),
        user_display,
        request_id,
        indent_body(&body_str)
    );

    info!("{}", request_log);

    // 重建请求
    let request = Request::from_parts(parts, axum::body::Body::from(bytes));

    // 处理请求
    let response = next.run(request).await;

    // 计算请求耗时
    let duration = start.elapsed();
    let status = response.status();

    // 根据状态码选择日志级别和状态标识
    let (status_icon, status_text) = if status.is_server_error() {
        ("❌", "Server Error")
    } else if status.is_client_error() {
        ("⚠️", "Client Error")
    } else {
        ("✓", "Success")
    };

    // 构建Footer
    let footer = format!("###################################{}###########", uri.path());

    // 构建响应日志
    let response_log = format!(
        "\n└─ 📤 RESPONSE [{}]\n\
            │  Method: {} {}\n\
            │  Handler: {}\n\
            │  Duration: {}ms\n\
            │  Client: {} | User: {}\n\
            │  Request ID: {}\n\
            └─ {} {}\n\
            \n\
            {}",
        status.as_u16(),
        method,
        uri,
        matched_path,
        duration.as_millis(),
        addr.ip(),
        user_display,
        request_id,
        status_icon,
        status_text,
        footer
    );

    // 根据状态码选择日志级别
    if status.is_server_error() {
        tracing::error!("{}", response_log);
    } else if status.is_client_error() {
        tracing::warn!("{}", response_log);
    } else {
        tracing::info!("{}", response_log);
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
    response.headers_mut().insert("X-Request-ID", request_id.parse().unwrap());

    response
}

/// 辅助函数：为请求体添加缩进，使其在日志中更易读
fn indent_body(body: &str) -> String {
    body.lines()
        .map(|line| format!("         │    {}", line))
        .collect::<Vec<_>>()
        .join("\n")
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
        use axum::{response::IntoResponse, routing::get, Router};
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
        let request = HttpRequest::builder().uri("/").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();

        // 验证响应状态码
        assert_eq!(response.status(), StatusCode::OK);

        // 验证响应头中包含X-Request-ID
        assert!(response.headers().contains_key("X-Request-ID"));
    }
}
