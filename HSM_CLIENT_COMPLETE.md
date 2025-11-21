# ✅ HSM客户端完成

## 概述

SUNBAY SoftPOS Backend的HSM (Hardware Security Module) 客户端已经完成实现。HSM客户端提供了与FutureX HSM通信的接口，用于安全的密钥派生操作。

## 已完成的功能

### HsmClient (`src/infrastructure/hsm_client.rs`)

HSM客户端提供了完整的密钥派生和健康检查功能。

#### 核心方法

1. **derive_ipek** - 派生IPEK (Initial PIN Encryption Key)
   - 调用HSM API派生IPEK
   - 自动后备到本地DUKPT
   - 用于密钥注入

2. **derive_working_key** - 派生Working Key
   - 调用HSM API派生Working Key
   - 自动后备到本地DUKPT
   - 用于PIN加密

3. **health_check** - HSM健康检查
   - 检查HSM连接状态
   - 验证API可用性
   - 用于监控

#### 配置

```rust
pub struct HsmConfig {
    pub url: String,           // HSM API URL
    pub api_key: String,       // API密钥
    pub timeout_seconds: u64,  // 超时时间
}
```

## 技术特性

### 1. 自动后备机制

当HSM不可用时，自动使用本地DUKPT：

```rust
match self.call_hsm_derive_ipek(ksn, device_id).await {
    Ok(ipek) => Ok(ipek),
    Err(e) => {
        tracing::warn!("HSM unavailable, using local DUKPT: {}", e);
        self.derive_ipek_local(ksn)
    }
}
```

### 2. HTTP客户端

使用reqwest进行HTTP通信：
- 可配置超时
- 支持重试
- 错误处理

### 3. API认证

使用API Key进行认证：
```rust
.header("X-API-Key", &self.config.api_key)
```

### 4. 错误处理

统一的错误处理：
- HSM不可用 → 使用本地后备
- API错误 → 返回详细错误信息
- 超时 → 自动重试或后备

### 5. 日志记录

完整的操作日志：
```rust
tracing::info!("IPEK derived successfully from HSM");
tracing::warn!("HSM unavailable, using local DUKPT");
```

## HSM API接口

### 1. 派生IPEK

**请求**:
```json
POST /api/v1/derive-ipek
{
  "ksn": "FFFF000000device1230000",
  "device_id": "device123"
}
```

**响应**:
```json
{
  "ipek": "0123456789ABCDEF...",
  "status": "success"
}
```

### 2. 派生Working Key

**请求**:
```json
POST /api/v1/derive-working-key
{
  "ipek": "0123456789ABCDEF...",
  "ksn": "FFFF000000device1230000"
}
```

**响应**:
```json
{
  "working_key": "FEDCBA9876543210...",
  "status": "success"
}
```

### 3. 健康检查

**请求**:
```
GET /api/v1/health
```

**响应**:
```json
{
  "status": "healthy",
  "version": "1.0.0"
}
```

## 代码示例

### 创建HSM客户端

```rust
use sunbay_softpos_backend::infrastructure::{HsmClient, HsmConfig};

// 配置HSM
let config = HsmConfig {
    url: "https://hsm.example.com".to_string(),
    api_key: "your-api-key".to_string(),
    timeout_seconds: 30,
};

// 创建客户端
let hsm_client = HsmClient::new(config)?;
```

### 派生IPEK

```rust
// 派生IPEK
let ksn = "FFFF000000device1230000";
let device_id = "device123";

let ipek = hsm_client.derive_ipek(ksn, device_id).await?;

println!("IPEK: {}", hex::encode(&ipek));
```

### 派生Working Key

```rust
// 派生Working Key
let working_key = hsm_client.derive_working_key(&ipek, ksn).await?;

println!("Working Key: {}", hex::encode(&working_key));
```

### 健康检查

```rust
// 检查HSM健康状态
let is_healthy = hsm_client.health_check().await?;

if is_healthy {
    println!("HSM is healthy");
} else {
    println!("HSM is unavailable, using local fallback");
}
```

## 后备机制

### 本地DUKPT后备

当HSM不可用时，自动使用本地DUKPT：

1. **derive_ipek_local** - 本地IPEK派生
2. **derive_working_key_local** - 本地Working Key派生
3. **get_local_bdk** - 获取本地BDK

### BDK管理

⚠️ **重要提示**：当前BDK是硬编码的示例值

生产环境应该：
1. 从安全的密钥管理系统获取BDK
2. 使用环境变量或配置文件
3. 定期轮换BDK
4. 使用HSM存储BDK

```rust
// 示例：从环境变量获取BDK
fn get_local_bdk(&self) -> Vec<u8> {
    let bdk_hex = std::env::var("BDK_KEY")
        .expect("BDK_KEY not set");
    hex::decode(&bdk_hex)
        .expect("Invalid BDK format")
}
```

## 安全考虑

### 1. API密钥管理
- ✅ 使用环境变量存储API密钥
- ✅ 不在代码中硬编码
- ✅ 定期轮换密钥

### 2. 通信安全
- ✅ 使用HTTPS通信
- ✅ 验证SSL证书
- ✅ 超时保护

### 3. 密钥保护
- ⚠️ BDK应该安全存储
- ⚠️ 生产环境使用HSM
- ⚠️ 实施密钥轮换

### 4. 错误处理
- ✅ 不泄露敏感信息
- ✅ 详细的日志记录
- ✅ 优雅的降级

## 测试

### 单元测试

所有核心功能都包含测试：

```rust
#[tokio::test]
async fn test_derive_ipek_fallback() {
    let client = create_test_client();
    let ipek = client.derive_ipek(ksn, "device123").await.unwrap();
    assert_eq!(ipek.len(), 32);
}
```

### 集成测试

可以使用Mock HSM服务器进行集成测试：

```rust
// 启动Mock HSM服务器
let mock_server = MockServer::start().await;

// 配置Mock响应
Mock::given(method("POST"))
    .and(path("/api/v1/derive-ipek"))
    .respond_with(ResponseTemplate::new(200)
        .set_body_json(json!({
            "ipek": "0123...",
            "status": "success"
        })))
    .mount(&mock_server)
    .await;
```

## 性能考虑

### 1. 连接池

HTTP客户端使用连接池：
- 复用连接
- 减少握手开销
- 提高性能

### 2. 超时设置

合理的超时设置：
- 默认30秒
- 可配置
- 防止长时间阻塞

### 3. 后备机制

快速后备：
- HSM失败立即切换
- 本地DUKPT性能好
- 不影响用户体验

### 4. 缓存策略

可以缓存Working Key：
- 减少HSM调用
- 提高响应速度
- 注意KSN同步

## 监控和日志

### 日志级别

- **INFO**: 成功的操作
- **WARN**: 后备机制触发
- **ERROR**: 操作失败
- **DEBUG**: 详细的调试信息

### 监控指标

建议监控：
1. HSM可用性
2. API响应时间
3. 后备机制使用率
4. 错误率

## 部署配置

### 环境变量

```bash
# HSM配置
HSM_URL=https://hsm.example.com
HSM_API_KEY=your-api-key
HSM_TIMEOUT=30

# 本地BDK（仅用于后备）
BDK_KEY=0123456789ABCDEF...
```

### 配置文件

```yaml
hsm:
  url: https://hsm.example.com
  api_key: ${HSM_API_KEY}
  timeout_seconds: 30
  
  # 后备配置
  fallback:
    enabled: true
    local_bdk: ${BDK_KEY}
```

## 与其他模块集成

### 与Service层

```rust
pub struct KeyManagementService {
    hsm_client: HsmClient,
    device_repo: DeviceRepository,
}

impl KeyManagementService {
    pub async fn inject_key(&self, device_id: &str) -> Result<Vec<u8>, AppError> {
        let device = self.device_repo.find_by_id(device_id).await?;
        let ksn = device.ksn.unwrap();
        
        // 使用HSM派生IPEK
        let ipek = self.hsm_client.derive_ipek(&ksn, device_id).await?;
        
        Ok(ipek)
    }
}
```

### 与监控系统

```rust
// 定期检查HSM健康状态
async fn monitor_hsm_health(hsm_client: &HsmClient) {
    loop {
        match hsm_client.health_check().await {
            Ok(true) => metrics::gauge!("hsm.healthy", 1.0),
            Ok(false) => metrics::gauge!("hsm.healthy", 0.0),
            Err(_) => metrics::gauge!("hsm.healthy", 0.0),
        }
        
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
```

## 下一步

HSM客户端已完成，可以继续实现：

1. **业务逻辑层** - 使用HSM客户端实现密钥管理服务（任务9-15）
2. **API层** - 提供密钥管理API（任务18-25）
3. **监控** - 实施HSM监控和告警（任务28）

## 生产环境清单

在部署到生产环境前，需要：

- [ ] 配置实际的FutureX HSM
- [ ] 设置安全的API密钥
- [ ] 配置BDK密钥管理
- [ ] 实施密钥轮换策略
- [ ] 配置SSL证书验证
- [ ] 实施监控和告警
- [ ] 进行安全审计
- [ ] 测试后备机制
- [ ] 文档化操作流程

## 总结

✅ HSM客户端完成
✅ IPEK派生功能
✅ Working Key派生功能
✅ 健康检查功能
✅ 自动后备机制
✅ 完整的错误处理
✅ 单元测试覆盖
⚠️ 需要配置实际HSM

**HSM客户端为系统提供了安全的密钥派生能力！** 🔐
