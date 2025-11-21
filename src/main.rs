use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use sunbay_softpos_backend::{
    api::AppState,
    infrastructure::Config,
    utils::error::AppError,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    init_tracing();

    tracing::info!("Starting SUNBAY SoftPOS A/M-Backend");

    // 加载配置
    let config = Config::load()?;
    tracing::info!("Configuration loaded successfully");

    // 初始化应用状态（包含所有服务）
    let app_state = AppState::new(config.clone()).await?;
    tracing::info!("Application state initialized with all services");

    // 创建路由
    let app = create_router(app_state);

    // 启动服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// AppState现在在api模块中定义

/// 创建路由
fn create_router(state: AppState) -> Router {
    Router::new()
        // 健康检查端点
        .route("/health", get(health_check))
        // API v1
        .nest("/api/v1", api_v1_routes())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

/// API v1 路由
fn api_v1_routes() -> Router<AppState> {
    Router::new()
        .route("/devices", post(register_device))
        .route("/devices", get(list_devices))
}

/// 健康检查处理器
async fn health_check(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    match state.health_check().await {
        Ok(_) => Ok(axum::Json(serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))),
        Err(e) => {
            tracing::error!("Health check failed: {}", e);
            Ok(axum::Json(serde_json::json!({
                "status": "unhealthy",
                "error": e.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            })))
        }
    }
}

// 设备处理器将在handlers模块中实现
// 这里保留简单的占位符用于测试

/// 设备注册处理器（占位符）
async fn register_device(
    axum::extract::State(_state): axum::extract::State<AppState>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    Ok(axum::Json(serde_json::json!({
        "message": "Device registration endpoint - to be implemented"
    })))
}

/// 设备列表处理器（占位符）
async fn list_devices(
    axum::extract::State(_state): axum::extract::State<AppState>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    Ok(axum::Json(serde_json::json!({
        "message": "Device list endpoint - to be implemented",
        "devices": [],
        "total": 0
    })))
}

/// 初始化日志
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sunbay_softpos_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
