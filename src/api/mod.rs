pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod websocket;

use crate::{
    infrastructure::{Config, HsmClient},
    repositories::{
        AuditLogRepository, DeviceRepository, HealthCheckRepository, ThreatRepository,
        TransactionRepository, VersionRepository,
    },
    security::{DukptKeyDerivation, JwtService},
    services::{
        AuditService, DeviceService, HealthCheckService, KeyManagementService,
        ThreatDetectionService, TransactionService, VersionService,
    },
};
use redis::Client as RedisClient;
use sqlx::SqlitePool;
use std::sync::Arc;

pub use routes::create_router;
pub use websocket::{ConnectionPool, NotificationService};

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    // 基础设施
    pub config: Arc<Config>,
    pub db_pool: SqlitePool,
    pub redis_client: Option<RedisClient>,
    pub hsm_client: Option<HsmClient>,

    // WebSocket
    pub ws_pool: ConnectionPool,
    pub notification_service: Arc<NotificationService>,

    // 安全模块
    pub jwt_service: Arc<JwtService>,
    pub dukpt: Arc<DukptKeyDerivation>,

    // 服务
    pub device_service: Arc<DeviceService>,
    pub key_management_service: Arc<KeyManagementService>,
    pub transaction_service: Arc<TransactionService>,
    pub audit_service: Arc<AuditService>,
    pub health_check_service: Arc<HealthCheckService>,
    pub threat_detection_service: Arc<ThreatDetectionService>,
    pub version_service: Arc<VersionService>,
}

impl AppState {
    /// 创建新的应用状态
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        tracing::info!("Initializing application state");

        // 初始化数据库连接池
        let db_pool = SqlitePool::connect(&config.database.url).await?;

        // 运行数据库迁移
        sqlx::migrate!("./migrations").run(&db_pool).await?;

        // 初始化Redis客户端
        let redis_client = Some(RedisClient::open(config.redis.url.as_str())?);

        // 初始化HSM客户端
        let hsm_client = Some(HsmClient::new(config.hsm.clone())?);

        // 初始化安全模块
        let jwt_service = Arc::new(JwtService::new(
            config.jwt.secret.clone(),
            config.jwt.expiration_hours * 3600,
        ));

        let dukpt = Arc::new(DukptKeyDerivation::new(
            config.security.bdk.clone().into_bytes(),
        ));

        // 初始化Repositories
        let device_repo = DeviceRepository::new(db_pool.clone());
        let audit_repo = AuditLogRepository::new(db_pool.clone());
        let health_check_repo = HealthCheckRepository::new(db_pool.clone());
        let threat_repo = ThreatRepository::new(db_pool.clone());
        let transaction_repo = TransactionRepository::new(db_pool.clone());
        let version_repo = VersionRepository::new(db_pool.clone());

        // 初始化Services
        let device_service = Arc::new(DeviceService::new(
            device_repo.clone(),
            audit_repo.clone(),
            (*dukpt).clone(),
            hsm_client.clone(),
        ));

        let key_management_service = Arc::new(KeyManagementService::new(
            device_repo.clone(),
            audit_repo.clone(),
            (*dukpt).clone(),
            hsm_client.clone(),
        ));

        let transaction_service = Arc::new(TransactionService::new(
            transaction_repo.clone(),
            device_repo.clone(),
            audit_repo.clone(),
            (*dukpt).clone(),
            hsm_client.clone(),
        ));

        let audit_service = Arc::new(AuditService::new(audit_repo.clone()));

        let health_check_service = Arc::new(HealthCheckService::new(
            health_check_repo.clone(),
            device_repo.clone(),
            threat_repo.clone(),
            audit_repo.clone(),
        ));

        let threat_detection_service = Arc::new(ThreatDetectionService::new(
            threat_repo.clone(),
            device_repo.clone(),
            health_check_repo.clone(),
            audit_repo.clone(),
        ));

        let version_service = Arc::new(VersionService::new(
            version_repo.clone(),
            device_repo.clone(),
            audit_repo.clone(),
        ));

        // 初始化WebSocket连接池和通知服务
        let ws_pool = websocket::create_connection_pool();
        let notification_service = Arc::new(NotificationService::new(ws_pool.clone()));

        tracing::info!("Application state initialized successfully");

        Ok(Self {
            config: Arc::new(config),
            db_pool,
            redis_client,
            hsm_client,
            ws_pool,
            notification_service,
            jwt_service,
            dukpt,
            device_service,
            key_management_service,
            transaction_service,
            audit_service,
            health_check_service,
            threat_detection_service,
            version_service,
        })
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 检查数据库连接
        sqlx::query("SELECT 1").execute(&self.db_pool).await?;

        // 检查Redis连接（如果配置了）
        if let Some(ref redis_client) = self.redis_client {
            let mut conn = redis_client.get_connection()?;
            redis::cmd("PING").query::<String>(&mut conn)?;
        }

        // 检查HSM连接（如果配置了）
        if let Some(ref hsm_client) = self.hsm_client {
            hsm_client.health_check().await?;
        }

        Ok(())
    }
}
