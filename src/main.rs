use std::{net::SocketAddr, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use sunbay_softpos_backend::{
    api::{create_router, AppState},
    infrastructure::Config,
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
    let app_state = Arc::new(AppState::new(config.clone()).await?);
    tracing::info!("Application state initialized with all services");

    // 使用完整的路由定义（来自 routes.rs）
    let app = create_router(app_state);

    // 启动服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>()
    ).await?;

    Ok(())
}

// 路由现在由 api::create_router 提供（定义在 src/api/routes.rs）
// 所有的 handler 函数都在 src/api/handlers/ 目录中实现

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
