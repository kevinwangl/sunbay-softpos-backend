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
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}

// 路由现在由 api::create_router 提供（定义在 src/api/routes.rs）
// 所有的 handler 函数都在 src/api/handlers/ 目录中实现

/// 初始化日志
fn init_tracing() {
    use sunbay_softpos_backend::infrastructure::SqlxLogLayer;
    use tracing_subscriber::fmt::format::FmtSpan;
    use tracing_subscriber::Layer;

    // 应用层日志Filter：显示应用日志，但过滤掉SQLx日志
    // 我们需要确保即使设置了RUST_LOG，也强制禁用sqlx
    let env_log = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "sunbay_softpos_backend=info,tower_http=info".to_string());
    // 使用更具体的 sqlx::query=off 来确保屏蔽查询日志
    let app_filter =
        tracing_subscriber::EnvFilter::new(format!("{},sqlx=off,sqlx::query=off", env_log));

    // SQLx日志Filter：允许SQLx的Debug日志通过（因为SQLx通常在Debug级别记录查询）
    let sqlx_filter = tracing_subscriber::EnvFilter::new("sqlx::query=debug");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(false)
                .with_line_number(false)
                .with_level(true)
                .with_ansi(true)
                .with_span_events(FmtSpan::NONE)
                .pretty()
                .with_filter(app_filter),
        )
        .with(SqlxLogLayer.with_filter(sqlx_filter))
        .init();
}
