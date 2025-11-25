use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::utils::error::AppError;
use crate::infrastructure::config::HsmConfig;

/// HSM客户端
/// 
/// 注意：这是一个模拟实现，用于开发和测试
/// 在生产环境中，应该连接到实际的FutureX HSM
#[derive(Clone)]
pub struct HsmClient {
    config: HsmConfig,
    client: Client,
}

/// IPEK派生请求
#[derive(Debug, Serialize)]
struct DeriveIpekRequest {
    ksn: String,
    device_id: String,
}

/// IPEK派生响应
#[derive(Debug, Deserialize)]
struct DeriveIpekResponse {
    ipek: String,
    status: String,
}

/// Working Key派生请求
#[derive(Debug, Serialize)]
struct DeriveWorkingKeyRequest {
    ipek: String,
    ksn: String,
}

/// Working Key派生响应
#[derive(Debug, Deserialize)]
struct DeriveWorkingKeyResponse {
    working_key: String,
    status: String,
}

/// HSM健康检查响应
#[derive(Debug, Deserialize)]
struct HsmHealthResponse {
    status: String,
    version: String,
}

impl HsmClient {
    /// 创建新的HSM客户端
    pub fn new(config: HsmConfig) -> Result<Self, AppError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| AppError::InternalWithMessage(format!("Failed to create HTTP client: {}", e)))?;

        tracing::info!("HSM client initialized for URL: {}", config.base_url);

        Ok(Self { config, client })
    }

    /// 派生IPEK
    /// 
    /// 在实际环境中，这会调用HSM的API来派生IPEK
    /// 当前实现使用本地DUKPT作为后备
    pub async fn derive_ipek(&self, ksn: &str, device_id: &str) -> Result<Vec<u8>, AppError> {
        tracing::debug!("Deriving IPEK for device: {}, KSN: {}", device_id, ksn);

        // 尝试调用HSM API
        match self.call_hsm_derive_ipek(ksn, device_id).await {
            Ok(ipek) => {
                tracing::info!("IPEK derived successfully from HSM");
                Ok(ipek)
            }
            Err(e) => {
                tracing::warn!("HSM unavailable, using local DUKPT: {}", e);
                // 后备：使用本地DUKPT
                self.derive_ipek_local(ksn)
            }
        }
    }

    /// 派生Working Key
    /// 
    /// 在实际环境中，这会调用HSM的API来派生Working Key
    /// 当前实现使用本地DUKPT作为后备
    pub async fn derive_working_key(
        &self,
        ipek: &[u8],
        ksn: &str,
    ) -> Result<Vec<u8>, AppError> {
        tracing::debug!("Deriving Working Key for KSN: {}", ksn);

        // 尝试调用HSM API
        match self.call_hsm_derive_working_key(ipek, ksn).await {
            Ok(working_key) => {
                tracing::info!("Working Key derived successfully from HSM");
                Ok(working_key)
            }
            Err(e) => {
                tracing::warn!("HSM unavailable, using local DUKPT: {}", e);
                // 后备：使用本地DUKPT
                self.derive_working_key_local(ipek, ksn)
            }
        }
    }

    /// HSM健康检查
    pub async fn health_check(&self) -> Result<bool, AppError> {
        tracing::debug!("Performing HSM health check");

        match self.call_hsm_health_check().await {
            Ok(_) => {
                tracing::info!("HSM health check passed");
                Ok(true)
            }
            Err(e) => {
                tracing::warn!("HSM health check failed: {}", e);
                Ok(false)
            }
        }
    }

    // ========== 私有方法：HSM API调用 ==========

    /// 调用HSM API派生IPEK
    async fn call_hsm_derive_ipek(
        &self,
        ksn: &str,
        device_id: &str,
    ) -> Result<Vec<u8>, AppError> {
        let url = format!("{}/api/v1/derive-ipek", self.config.base_url);

        let request = DeriveIpekRequest {
            ksn: ksn.to_string(),
            device_id: device_id.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.config.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::External(format!("HSM API call failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::External(format!(
                "HSM API returned error: {}",
                response.status()
            )));
        }

        let hsm_response: DeriveIpekResponse = response
            .json()
            .await
            .map_err(|e| AppError::External(format!("Failed to parse HSM response: {}", e)))?;

        if hsm_response.status != "success" {
            return Err(AppError::External(format!(
                "HSM operation failed: {}",
                hsm_response.status
            )));
        }

        // 解码IPEK
        hex::decode(&hsm_response.ipek)
            .map_err(|e| AppError::External(format!("Invalid IPEK format from HSM: {}", e)))
    }

    /// 调用HSM API派生Working Key
    async fn call_hsm_derive_working_key(
        &self,
        ipek: &[u8],
        ksn: &str,
    ) -> Result<Vec<u8>, AppError> {
        let url = format!("{}/api/v1/derive-working-key", self.config.base_url);

        let request = DeriveWorkingKeyRequest {
            ipek: hex::encode(ipek),
            ksn: ksn.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.config.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::External(format!("HSM API call failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::External(format!(
                "HSM API returned error: {}",
                response.status()
            )));
        }

        let hsm_response: DeriveWorkingKeyResponse = response
            .json()
            .await
            .map_err(|e| AppError::External(format!("Failed to parse HSM response: {}", e)))?;

        if hsm_response.status != "success" {
            return Err(AppError::External(format!(
                "HSM operation failed: {}",
                hsm_response.status
            )));
        }

        // 解码Working Key
        hex::decode(&hsm_response.working_key).map_err(|e| {
            AppError::External(format!("Invalid Working Key format from HSM: {}", e))
        })
    }

    /// 调用HSM健康检查API
    async fn call_hsm_health_check(&self) -> Result<(), AppError> {
        let url = format!("{}/api/v1/health", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.config.api_key)
            .send()
            .await
            .map_err(|e| AppError::External(format!("HSM health check failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::External(format!(
                "HSM health check returned error: {}",
                response.status()
            )));
        }

        let _hsm_response: HsmHealthResponse = response
            .json()
            .await
            .map_err(|e| AppError::External(format!("Failed to parse HSM response: {}", e)))?;

        Ok(())
    }

    // ========== 私有方法：本地DUKPT后备 ==========

    /// 使用本地DUKPT派生IPEK（后备方案）
    fn derive_ipek_local(&self, ksn: &str) -> Result<Vec<u8>, AppError> {
        // 使用本地DUKPT服务
        // 注意：BDK应该从安全配置中获取
        let bdk = self.get_local_bdk();
        let dukpt = crate::security::DukptKeyDerivation::new(bdk);

        dukpt.derive_ipek(ksn)
    }

    /// 使用本地DUKPT派生Working Key（后备方案）
    fn derive_working_key_local(&self, ipek: &[u8], ksn: &str) -> Result<Vec<u8>, AppError> {
        let bdk = self.get_local_bdk();
        let dukpt = crate::security::DukptKeyDerivation::new(bdk);

        dukpt.derive_working_key(ipek, ksn)
    }

    /// 获取本地BDK（Base Derivation Key）
    /// 
    /// 注意：这是一个示例实现
    /// 在生产环境中，BDK应该：
    /// 1. 从安全的密钥管理系统获取
    /// 2. 使用HSM存储
    /// 3. 定期轮换
    fn get_local_bdk(&self) -> Vec<u8> {
        // 示例BDK - 在生产环境中应该从安全配置获取
        vec![
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54,
            0x32, 0x10,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_client() -> HsmClient {
        let config = HsmConfig {
            base_url: "http://localhost:8888".to_string(),
            api_key: "test-api-key".to_string(),
            timeout_seconds: 30,
        };

        HsmClient::new(config).unwrap()
    }

    #[tokio::test]
    async fn test_derive_ipek_fallback() {
        let client = create_test_client();
        // Valid 24-char hex KSN: FFFF000000 (10 chars) + 6465766963 (10 chars) + 0000 (4 chars) = 24 total
        let ksn = "FFFF00000064657669630000";

        // 由于没有实际的HSM，应该使用本地后备
        let ipek = client.derive_ipek(ksn, "device123").await.unwrap();

        assert_eq!(ipek.len(), 32); // SHA256 produces 32 bytes
    }

    #[tokio::test]
    async fn test_derive_working_key_fallback() {
        let client = create_test_client();
        // Valid 24-char hex KSN: FFFF000000 (10 chars) + 6465766963 (10 chars) + 0000 (4 chars) = 24 total
        let ksn = "FFFF00000064657669630000";

        let ipek = client.derive_ipek(ksn, "device123").await.unwrap();
        let working_key = client.derive_working_key(&ipek, ksn).await.unwrap();

        assert_eq!(working_key.len(), 32);
    }

    #[tokio::test]
    async fn test_health_check() {
        let client = create_test_client();

        // 由于没有实际的HSM，健康检查应该返回false
        let is_healthy = client.health_check().await.unwrap();

        assert!(!is_healthy);
    }
}
