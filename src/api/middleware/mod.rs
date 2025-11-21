pub mod auth;
pub mod logging;
pub mod metrics;
pub mod prometheus;
pub mod rate_limit;
pub mod tracing;

pub use auth::{
    auth_middleware, extract_claims, extract_role, extract_user_id, extract_username, has_role,
    optional_auth_middleware, require_role,
};
pub use logging::{
    error_logging_middleware, logging_middleware, request_id_middleware,
    slow_request_logging_middleware, structured_logging_middleware,
};
pub use metrics::{
    metrics_middleware, DeviceMetricType, DeviceMetrics, EndpointMetrics, MetricsCollector,
    RequestMetrics, TransactionMetricType, TransactionMetrics,
};
pub use prometheus::{metrics_handler, prometheus_middleware, PrometheusMetrics};
pub use rate_limit::{
    rate_limit_layer, rate_limit_middleware, user_rate_limit_middleware, RateLimitConfig,
    RateLimiter,
};
pub use tracing::{extract_trace_context, tracing_middleware, TraceContext};
