use axum::{
    extract::{ConnectInfo, Request},
    middleware::Next,
    response::Response,
};
use std::{
    net::SocketAddr,
    sync::Arc,
    time::Instant,
};
use tokio::sync::Mutex;
use std::collections::HashMap;

/// 请求指标
#[derive(Debug, Clone, Default)]
pub struct RequestMetrics {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数（2xx）
    pub successful_requests: u64,
    /// 客户端错误数（4xx）
    pub client_errors: u64,
    /// 服务器错误数（5xx）
    pub server_errors: u64,
    /// 总响应时间（毫秒）
    pub total_response_time_ms: u64,
    /// 最小响应时间（毫秒）
    pub min_response_time_ms: u64,
    /// 最大响应时间（毫秒）
    pub max_response_time_ms: u64,
}

impl RequestMetrics {
    /// 记录请求
    pub fn record_request(&mut self, status_code: u16, duration_ms: u64) {
        self.total_requests += 1;
        self.total_response_time_ms += duration_ms;

        // 更新最小/最大响应时间
        if self.min_response_time_ms == 0 || duration_ms < self.min_response_time_ms {
            self.min_response_time_ms = duration_ms;
        }
        if duration_ms > self.max_response_time_ms {
            self.max_response_time_ms = duration_ms;
        }

        // 根据状态码分类
        match status_code {
            200..=299 => self.successful_requests += 1,
            400..=499 => self.client_errors += 1,
            500..=599 => self.server_errors += 1,
            _ => {}
        }
    }

    /// 获取平均响应时间
    pub fn avg_response_time_ms(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.total_response_time_ms as f64 / self.total_requests as f64
        }
    }

    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    /// 获取错误率
    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            ((self.client_errors + self.server_errors) as f64 / self.total_requests as f64) * 100.0
        }
    }
}

/// 端点指标
#[derive(Debug, Clone, Default)]
pub struct EndpointMetrics {
    /// 按端点分组的指标
    pub endpoints: HashMap<String, RequestMetrics>,
}

impl EndpointMetrics {
    /// 记录端点请求
    pub fn record_endpoint_request(&mut self, endpoint: String, status_code: u16, duration_ms: u64) {
        let metrics = self.endpoints.entry(endpoint).or_default();
        metrics.record_request(status_code, duration_ms);
    }

    /// 获取端点指标
    pub fn get_endpoint_metrics(&self, endpoint: &str) -> Option<&RequestMetrics> {
        self.endpoints.get(endpoint)
    }

    /// 获取所有端点指标
    pub fn get_all_metrics(&self) -> &HashMap<String, RequestMetrics> {
        &self.endpoints
    }
}

/// 设备指标
#[derive(Debug, Clone, Default)]
pub struct DeviceMetrics {
    /// 设备注册数
    pub device_registrations: u64,
    /// 设备审批数
    pub device_approvals: u64,
    /// 设备拒绝数
    pub device_rejections: u64,
    /// 设备暂停数
    pub device_suspensions: u64,
    /// 设备吊销数
    pub device_revocations: u64,
}

impl DeviceMetrics {
    /// 记录设备注册
    pub fn record_registration(&mut self) {
        self.device_registrations += 1;
    }

    /// 记录设备审批
    pub fn record_approval(&mut self) {
        self.device_approvals += 1;
    }

    /// 记录设备拒绝
    pub fn record_rejection(&mut self) {
        self.device_rejections += 1;
    }

    /// 记录设备暂停
    pub fn record_suspension(&mut self) {
        self.device_suspensions += 1;
    }

    /// 记录设备吊销
    pub fn record_revocation(&mut self) {
        self.device_revocations += 1;
    }
}

/// 交易指标
#[derive(Debug, Clone, Default)]
pub struct TransactionMetrics {
    /// 交易鉴证数
    pub transaction_attestations: u64,
    /// 交易处理数
    pub transaction_processes: u64,
    /// 交易成功数
    pub successful_transactions: u64,
    /// 交易失败数
    pub failed_transactions: u64,
}

impl TransactionMetrics {
    /// 记录交易鉴证
    pub fn record_attestation(&mut self) {
        self.transaction_attestations += 1;
    }

    /// 记录交易处理
    pub fn record_process(&mut self, success: bool) {
        self.transaction_processes += 1;
        if success {
            self.successful_transactions += 1;
        } else {
            self.failed_transactions += 1;
        }
    }

    /// 获取交易成功率
    pub fn success_rate(&self) -> f64 {
        if self.transaction_processes == 0 {
            0.0
        } else {
            (self.successful_transactions as f64 / self.transaction_processes as f64) * 100.0
        }
    }
}

/// 应用指标收集器
#[derive(Clone)]
pub struct MetricsCollector {
    /// 全局请求指标
    pub request_metrics: Arc<Mutex<RequestMetrics>>,
    /// 端点指标
    pub endpoint_metrics: Arc<Mutex<EndpointMetrics>>,
    /// 设备指标
    pub device_metrics: Arc<Mutex<DeviceMetrics>>,
    /// 交易指标
    pub transaction_metrics: Arc<Mutex<TransactionMetrics>>,
}

impl MetricsCollector {
    /// 创建新的指标收集器
    pub fn new() -> Self {
        Self {
            request_metrics: Arc::new(Mutex::new(RequestMetrics::default())),
            endpoint_metrics: Arc::new(Mutex::new(EndpointMetrics::default())),
            device_metrics: Arc::new(Mutex::new(DeviceMetrics::default())),
            transaction_metrics: Arc::new(Mutex::new(TransactionMetrics::default())),
        }
    }

    /// 记录请求指标
    pub async fn record_request(&self, status_code: u16, duration_ms: u64) {
        let mut metrics = self.request_metrics.lock().await;
        metrics.record_request(status_code, duration_ms);
    }

    /// 记录端点请求指标
    pub async fn record_endpoint_request(&self, endpoint: String, status_code: u16, duration_ms: u64) {
        let mut metrics = self.endpoint_metrics.lock().await;
        metrics.record_endpoint_request(endpoint, status_code, duration_ms);
    }

    /// 记录设备指标
    pub async fn record_device_metric(&self, metric_type: DeviceMetricType) {
        let mut metrics = self.device_metrics.lock().await;
        match metric_type {
            DeviceMetricType::Registration => metrics.record_registration(),
            DeviceMetricType::Approval => metrics.record_approval(),
            DeviceMetricType::Rejection => metrics.record_rejection(),
            DeviceMetricType::Suspension => metrics.record_suspension(),
            DeviceMetricType::Revocation => metrics.record_revocation(),
        }
    }

    /// 记录交易指标
    pub async fn record_transaction_metric(&self, metric_type: TransactionMetricType) {
        let mut metrics = self.transaction_metrics.lock().await;
        match metric_type {
            TransactionMetricType::Attestation => metrics.record_attestation(),
            TransactionMetricType::Process { success } => metrics.record_process(success),
        }
    }

    /// 获取全局请求指标
    pub async fn get_request_metrics(&self) -> RequestMetrics {
        self.request_metrics.lock().await.clone()
    }

    /// 获取端点指标
    pub async fn get_endpoint_metrics(&self) -> EndpointMetrics {
        self.endpoint_metrics.lock().await.clone()
    }

    /// 获取设备指标
    pub async fn get_device_metrics(&self) -> DeviceMetrics {
        self.device_metrics.lock().await.clone()
    }

    /// 获取交易指标
    pub async fn get_transaction_metrics(&self) -> TransactionMetrics {
        self.transaction_metrics.lock().await.clone()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// 设备指标类型
pub enum DeviceMetricType {
    Registration,
    Approval,
    Rejection,
    Suspension,
    Revocation,
}

/// 交易指标类型
pub enum TransactionMetricType {
    Attestation,
    Process { success: bool },
}

/// 指标收集中间件
///
/// 收集HTTP请求的指标信息
pub async fn metrics_middleware(
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    collector: axum::extract::State<Arc<MetricsCollector>>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let endpoint = format!("{} {}", method, uri.path());

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    let status_code = response.status().as_u16();
    let duration_ms = duration.as_millis() as u64;

    // 记录全局指标
    collector.record_request(status_code, duration_ms).await;

    // 记录端点指标
    collector.record_endpoint_request(endpoint, status_code, duration_ms).await;

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_metrics() {
        let mut metrics = RequestMetrics::default();

        metrics.record_request(200, 100);
        metrics.record_request(404, 50);
        metrics.record_request(500, 200);

        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.client_errors, 1);
        assert_eq!(metrics.server_errors, 1);
        assert_eq!(metrics.min_response_time_ms, 50);
        assert_eq!(metrics.max_response_time_ms, 200);
        assert_eq!(metrics.avg_response_time_ms(), 116.66666666666667);
    }

    #[test]
    fn test_endpoint_metrics() {
        let mut metrics = EndpointMetrics::default();

        metrics.record_endpoint_request("GET /api/devices".to_string(), 200, 100);
        metrics.record_endpoint_request("GET /api/devices".to_string(), 200, 150);
        metrics.record_endpoint_request("POST /api/devices".to_string(), 201, 200);

        let get_metrics = metrics.get_endpoint_metrics("GET /api/devices").unwrap();
        assert_eq!(get_metrics.total_requests, 2);
        assert_eq!(get_metrics.successful_requests, 2);

        let post_metrics = metrics.get_endpoint_metrics("POST /api/devices").unwrap();
        assert_eq!(post_metrics.total_requests, 1);
        assert_eq!(post_metrics.successful_requests, 1);
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        collector.record_request(200, 100).await;
        collector.record_request(404, 50).await;

        let metrics = collector.get_request_metrics().await;
        assert_eq!(metrics.total_requests, 2);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.client_errors, 1);
    }

    #[tokio::test]
    async fn test_device_metrics() {
        let collector = MetricsCollector::new();

        collector.record_device_metric(DeviceMetricType::Registration).await;
        collector.record_device_metric(DeviceMetricType::Approval).await;
        collector.record_device_metric(DeviceMetricType::Rejection).await;

        let metrics = collector.get_device_metrics().await;
        assert_eq!(metrics.device_registrations, 1);
        assert_eq!(metrics.device_approvals, 1);
        assert_eq!(metrics.device_rejections, 1);
    }

    #[tokio::test]
    async fn test_transaction_metrics() {
        let collector = MetricsCollector::new();

        collector.record_transaction_metric(TransactionMetricType::Attestation).await;
        collector.record_transaction_metric(TransactionMetricType::Process { success: true }).await;
        collector.record_transaction_metric(TransactionMetricType::Process { success: false }).await;

        let metrics = collector.get_transaction_metrics().await;
        assert_eq!(metrics.transaction_attestations, 1);
        assert_eq!(metrics.transaction_processes, 2);
        assert_eq!(metrics.successful_transactions, 1);
        assert_eq!(metrics.failed_transactions, 1);
        assert_eq!(metrics.success_rate(), 50.0);
    }
}
