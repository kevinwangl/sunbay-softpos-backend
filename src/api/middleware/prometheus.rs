use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use prometheus::{
    register_counter_vec, register_gauge_vec, register_histogram_vec, CounterVec, Encoder,
    GaugeVec, HistogramVec, TextEncoder,
};
use std::{net::SocketAddr, sync::Arc, time::Instant};
use tokio::sync::OnceCell;

/// Prometheus指标收集器
#[derive(Clone)]
pub struct PrometheusMetrics {
    /// HTTP请求总数
    pub http_requests_total: CounterVec,
    /// HTTP请求持续时间
    pub http_request_duration_seconds: HistogramVec,
    /// 活跃连接数
    pub active_connections: GaugeVec,
    /// 设备指标
    pub devices_total: GaugeVec,
    /// 交易指标
    pub transactions_total: CounterVec,
    /// 威胁指标
    pub threats_total: CounterVec,
    /// 健康检查指标
    pub health_checks_total: CounterVec,
    /// 密钥操作指标
    pub key_operations_total: CounterVec,
    /// 版本管理指标
    pub version_operations_total: CounterVec,
    /// WebSocket连接指标
    pub websocket_connections: GaugeVec,
    /// 系统资源指标
    pub system_info: GaugeVec,
}

static METRICS: OnceCell<Arc<PrometheusMetrics>> = OnceCell::const_new();

impl PrometheusMetrics {
    /// 初始化Prometheus指标
    pub fn init() -> Result<Arc<Self>, Box<dyn std::error::Error + Send + Sync>> {
        let http_requests_total = register_counter_vec!(
            "sunbay_http_requests_total",
            "Total number of HTTP requests",
            &["method", "endpoint", "status"]
        )?;

        let http_request_duration_seconds = register_histogram_vec!(
            "sunbay_http_request_duration_seconds",
            "HTTP request duration in seconds",
            &["method", "endpoint"],
            vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
        )?;

        let active_connections = register_gauge_vec!(
            "sunbay_active_connections",
            "Number of active connections",
            &["type"]
        )?;

        let devices_total = register_gauge_vec!(
            "sunbay_devices_total",
            "Total number of devices by status",
            &["status"]
        )?;

        let transactions_total = register_counter_vec!(
            "sunbay_transactions_total",
            "Total number of transactions",
            &["type", "status", "device_mode"]
        )?;

        let threats_total = register_counter_vec!(
            "sunbay_threats_total",
            "Total number of threats detected",
            &["type", "severity", "status"]
        )?;

        let health_checks_total = register_counter_vec!(
            "sunbay_health_checks_total",
            "Total number of health checks",
            &["device_id", "result"]
        )?;

        let key_operations_total = register_counter_vec!(
            "sunbay_key_operations_total",
            "Total number of key operations",
            &["operation", "key_type", "status"]
        )?;

        let version_operations_total = register_counter_vec!(
            "sunbay_version_operations_total",
            "Total number of version operations",
            &["operation", "update_type", "status"]
        )?;

        let websocket_connections = register_gauge_vec!(
            "sunbay_websocket_connections",
            "Number of WebSocket connections",
            &["status"]
        )?;

        let system_info = register_gauge_vec!(
            "sunbay_system_info",
            "System information",
            &["component", "version"]
        )?;

        Ok(Arc::new(Self {
            http_requests_total,
            http_request_duration_seconds,
            active_connections,
            devices_total,
            transactions_total,
            threats_total,
            health_checks_total,
            key_operations_total,
            version_operations_total,
            websocket_connections,
            system_info,
        }))
    }

    /// 获取全局指标实例
    pub async fn global() -> Arc<Self> {
        METRICS
            .get_or_init(|| async {
                Self::init().expect("Failed to initialize Prometheus metrics")
            })
            .await
            .clone()
    }

    /// 记录HTTP请求
    pub fn record_http_request(&self, method: &str, endpoint: &str, status: u16, duration: f64) {
        self.http_requests_total
            .with_label_values(&[method, endpoint, &status.to_string()])
            .inc();
        
        self.http_request_duration_seconds
            .with_label_values(&[method, endpoint])
            .observe(duration);
    }

    /// 更新设备统计
    pub fn update_device_count(&self, status: &str, count: f64) {
        self.devices_total
            .with_label_values(&[status])
            .set(count);
    }

    /// 记录交易
    pub fn record_transaction(&self, transaction_type: &str, status: &str, device_mode: &str) {
        self.transactions_total
            .with_label_values(&[transaction_type, status, device_mode])
            .inc();
    }

    /// 记录威胁
    pub fn record_threat(&self, threat_type: &str, severity: &str, status: &str) {
        self.threats_total
            .with_label_values(&[threat_type, severity, status])
            .inc();
    }

    /// 记录健康检查
    pub fn record_health_check(&self, device_id: &str, result: &str) {
        self.health_checks_total
            .with_label_values(&[device_id, result])
            .inc();
    }

    /// 记录密钥操作
    pub fn record_key_operation(&self, operation: &str, key_type: &str, status: &str) {
        self.key_operations_total
            .with_label_values(&[operation, key_type, status])
            .inc();
    }

    /// 记录版本操作
    pub fn record_version_operation(&self, operation: &str, update_type: &str, status: &str) {
        self.version_operations_total
            .with_label_values(&[operation, update_type, status])
            .inc();
    }

    /// 更新WebSocket连接数
    pub fn update_websocket_connections(&self, status: &str, count: f64) {
        self.websocket_connections
            .with_label_values(&[status])
            .set(count);
    }

    /// 设置系统信息
    pub fn set_system_info(&self, component: &str, version: &str) {
        self.system_info
            .with_label_values(&[component, version])
            .set(1.0);
    }

    /// 更新活跃连接数
    pub fn update_active_connections(&self, connection_type: &str, count: f64) {
        self.active_connections
            .with_label_values(&[connection_type])
            .set(count);
    }
}

/// Prometheus指标中间件
pub async fn prometheus_middleware(
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    
    // 简化路径，移除动态参数
    let endpoint = simplify_path(&path);
    
    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed().as_secs_f64();
    
    let status = response.status().as_u16();
    
    // 记录指标
    if let Ok(metrics) = PrometheusMetrics::init() {
        metrics.record_http_request(&method, &endpoint, status, duration);
    }
    
    response
}

/// 简化路径，移除动态参数
fn simplify_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    let mut simplified = Vec::new();
    
    for part in parts {
        if part.is_empty() {
            continue;
        }
        
        // 如果是UUID或数字ID，替换为占位符
        if is_uuid(part) || part.parse::<i64>().is_ok() {
            simplified.push(":id");
        } else {
            simplified.push(part);
        }
    }
    
    format!("/{}", simplified.join("/"))
}

/// 检查是否是UUID
fn is_uuid(s: &str) -> bool {
    s.len() == 36 && s.chars().filter(|c| *c == '-').count() == 4
}

/// Prometheus指标端点处理器
pub async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    
    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to encode metrics: {}", e),
        )
            .into_response();
    }
    
    match String::from_utf8(buffer) {
        Ok(metrics) => (StatusCode::OK, metrics).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to convert metrics to string: {}", e),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_path() {
        assert_eq!(simplify_path("/api/v1/devices"), "/api/v1/devices");
        assert_eq!(
            simplify_path("/api/v1/devices/123"),
            "/api/v1/devices/:id"
        );
        assert_eq!(
            simplify_path("/api/v1/devices/550e8400-e29b-41d4-a716-446655440000"),
            "/api/v1/devices/:id"
        );
    }

    #[test]
    fn test_is_uuid() {
        assert!(is_uuid("550e8400-e29b-41d4-a716-446655440000"));
        assert!(!is_uuid("123"));
        assert!(!is_uuid("not-a-uuid"));
    }

    #[tokio::test]
    async fn test_prometheus_metrics_init() {
        let metrics = PrometheusMetrics::init();
        assert!(metrics.is_ok());
    }
}
