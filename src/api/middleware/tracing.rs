use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::{info_span, Instrument};
use uuid::Uuid;

/// 分布式追踪中间件
/// 为每个请求创建一个追踪span，并传播追踪上下文
pub async fn tracing_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // 从请求头中提取或生成trace_id
    let trace_id = request
        .headers()
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // 从请求头中提取或生成span_id
    let span_id = request
        .headers()
        .get("x-span-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // 提取parent_span_id（如果存在）
    let parent_span_id = request
        .headers()
        .get("x-parent-span-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 将trace_id和span_id存储到请求扩展中，供后续使用
    request.extensions_mut().insert(TraceContext {
        trace_id: trace_id.clone(),
        span_id: span_id.clone(),
        parent_span_id: parent_span_id.clone(),
    });

    // 创建span
    let span = if let Some(parent_id) = parent_span_id {
        info_span!(
            "http_request",
            trace_id = %trace_id,
            span_id = %span_id,
            parent_span_id = %parent_id,
            method = %request.method(),
            uri = %request.uri(),
        )
    } else {
        info_span!(
            "http_request",
            trace_id = %trace_id,
            span_id = %span_id,
            method = %request.method(),
            uri = %request.uri(),
        )
    };

    // 在span上下文中执行请求
    async move {
        let mut response = next.run(request).await;
        
        // 将trace_id添加到响应头中
        response.headers_mut().insert(
            "x-trace-id",
            trace_id.parse().unwrap(),
        );
        response.headers_mut().insert(
            "x-span-id",
            span_id.parse().unwrap(),
        );
        
        response
    }
    .instrument(span)
    .await
}

/// 追踪上下文
#[derive(Debug, Clone)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
}

impl TraceContext {
    /// 创建新的追踪上下文
    pub fn new() -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
        }
    }

    /// 创建子span
    pub fn create_child_span(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: Some(self.span_id.clone()),
        }
    }

    /// 获取trace_id
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// 获取span_id
    pub fn span_id(&self) -> &str {
        &self.span_id
    }

    /// 获取parent_span_id
    pub fn parent_span_id(&self) -> Option<&str> {
        self.parent_span_id.as_deref()
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 从请求扩展中提取追踪上下文
pub fn extract_trace_context(request: &Request) -> Option<TraceContext> {
    request.extensions().get::<TraceContext>().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_context_new() {
        let ctx = TraceContext::new();
        assert!(!ctx.trace_id.is_empty());
        assert!(!ctx.span_id.is_empty());
        assert!(ctx.parent_span_id.is_none());
    }

    #[test]
    fn test_trace_context_create_child_span() {
        let parent = TraceContext::new();
        let child = parent.create_child_span();
        
        assert_eq!(child.trace_id, parent.trace_id);
        assert_ne!(child.span_id, parent.span_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id.clone()));
    }
}
