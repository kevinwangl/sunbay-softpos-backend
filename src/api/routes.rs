use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::api::{
    handlers, middleware as api_middleware, websocket::websocket_handler, AppState, 
    middleware::{MetricsCollector, metrics_handler},
};

/// 创建应用路由
pub fn create_router(state: Arc<AppState>) -> Router {
    // 创建指标收集器
    let metrics_collector = Arc::new(MetricsCollector::new());

    // 创建速率限制器
    let rate_limiter = api_middleware::rate_limit_layer(api_middleware::RateLimitConfig::default());

    // 配置CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 公开路由（不需要认证）
    let public_routes = Router::new()
        // 健康检查
        .route("/health/check", get(handlers::health_check))
        // Prometheus指标
        .route("/metrics", get(metrics_handler))
        // 认证
        .route("/auth/login", post(handlers::login))
        .route("/auth/refresh", post(handlers::refresh_token))
        .route("/auth/verify", post(handlers::verify_token))
        // 设备注册（公开）
        .route("/devices/register", post(handlers::register_device))
        // 威胁上报（公开，设备端调用）
        .route("/threats/report", post(handlers::report_threat))
        // WebSocket连接
        .route("/ws", get(websocket_handler));

    // 受保护的路由（需要认证）
    let protected_routes = Router::new()
        // 认证相关
        .route("/auth/logout", post(handlers::logout))
        .route("/auth/me", get(handlers::get_current_user))
        // 仪表盘
        .route(
            "/dashboard/health-overview",
            get(handlers::get_dashboard_health_overview),
        )
        // 设备管理
        .route("/devices", get(handlers::list_devices))
        .route("/devices/statistics", get(handlers::get_device_statistics))
        .route("/devices/:device_id", get(handlers::get_device))
        .route(
            "/devices/:device_id/approve",
            post(handlers::approve_device),
        )
        .route(
            "/devices/:device_id/reject",
            post(handlers::reject_device),
        )
        .route(
            "/devices/:device_id/suspend",
            post(handlers::suspend_device),
        )
        .route(
            "/devices/:device_id/resume",
            post(handlers::resume_device),
        )
        .route(
            "/devices/:device_id/revoke",
            post(handlers::revoke_device),
        )
        // 密钥管理
        .route("/keys/inject", post(handlers::inject_key))
        .route("/keys/:device_id/status", get(handlers::get_key_status))
        .route("/keys/:device_id/update", post(handlers::update_key))
        .route(
            "/keys/:device_id/check-update",
            get(handlers::check_key_update_needed),
        )
        .route(
            "/keys/devices-needing-update",
            get(handlers::get_devices_needing_key_update),
        )
        // 健康检查
        .route("/health/submit", post(handlers::submit_health_check))
        .route("/health/checks", get(handlers::list_health_checks))
        .route(
            "/health/:device_id/overview",
            get(handlers::get_health_overview),
        )
        .route(
            "/health/:device_id/initial-check",
            post(handlers::perform_initial_check),
        )
        .route("/health/statistics", get(handlers::get_health_statistics))
        // 威胁管理
        .route("/threats", get(handlers::list_threats))
        .route("/threats/statistics", get(handlers::get_threat_statistics))
        .route("/threats/:threat_id", get(handlers::get_threat))
        .route(
            "/threats/:threat_id/resolve",
            post(handlers::resolve_threat),
        )
        .route(
            "/threats/device/:device_id/history",
            get(handlers::get_device_threat_history),
        )
        // 交易管理
        .route(
            "/transactions/attest",
            post(handlers::attest_transaction),
        )
        .route(
            "/transactions/process",
            post(handlers::process_transaction),
        )
        .route("/transactions", get(handlers::list_transactions))
        .route(
            "/transactions/statistics",
            get(handlers::get_transaction_statistics),
        )
        .route(
            "/transactions/:transaction_id",
            get(handlers::get_transaction),
        )
        .route(
            "/transactions/device/:device_id/history",
            get(handlers::get_device_transaction_history),
        )
        // PINPad模式
        .route("/pinpad/attest", post(handlers::attest_pinpad))
        .route("/pinpad/logs", get(handlers::list_pin_encryption_logs))
        .route(
            "/pinpad/device/:device_id/statistics",
            get(handlers::get_device_pin_statistics),
        )
        .route(
            "/pinpad/device/:device_id/status",
            get(handlers::get_pinpad_device_status),
        )
        // 版本管理
        .route("/versions", post(handlers::create_version))
        .route("/versions", get(handlers::list_versions))
        .route("/versions/statistics", get(handlers::get_version_statistics))
        .route(
            "/versions/compatibility",
            get(handlers::get_compatibility_matrix),
        )
        .route(
            "/versions/outdated-devices",
            get(handlers::get_outdated_devices),
        )
        .route(
            "/versions/update-dashboard",
            get(handlers::get_update_dashboard),
        )
        .route("/versions/push", post(handlers::create_push_task))
        .route("/versions/push", get(handlers::list_push_tasks))
        .route("/versions/push/:task_id", get(handlers::get_push_task))
        .route("/versions/:version_id", get(handlers::get_version))
        .route("/versions/:version_id", put(handlers::update_version))
        .route(
            "/versions/available/:device_id",
            get(handlers::get_available_version),
        )
        // 审计日志
        .route("/audit/logs", get(handlers::list_logs))
        .route("/audit/logs/:log_id", get(handlers::get_log))
        .route("/audit/statistics", get(handlers::get_audit_statistics))
        .route("/audit/export", get(handlers::export_logs))
        .route(
            "/audit/device/:device_id/logs",
            get(handlers::get_device_logs),
        )
        .route(
            "/audit/operator/:operator_id/logs",
            get(handlers::get_operator_logs),
        )
        // 应用认证中间件
        .layer(middleware::from_fn_with_state(
            state.clone(),
            api_middleware::auth_middleware,
        ));

    // API v1路由
    let api_v1 = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        // 应用日志中间件
        .layer(middleware::from_fn(api_middleware::logging_middleware))
        // 应用请求ID中间件
        .layer(middleware::from_fn(api_middleware::request_id_middleware))
        // 应用指标收集中间件
        .layer(middleware::from_fn_with_state(
            metrics_collector.clone(),
            api_middleware::metrics_middleware,
        ))
        // 应用速率限制中间件
        .layer(middleware::from_fn_with_state(
            rate_limiter,
            api_middleware::rate_limit_middleware,
        ));

    // 根路由 - 添加根健康检查端点
    Router::new()
        .route("/health", get(|| async {
            use axum::Json;
            Json(serde_json::json!({
                "status": "ok",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }))
        .nest("/api/v1", api_v1)
        .layer(cors)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_router() {
        // 简单的编译时测试
        // 实际测试需要完整的AppState
    }
}
