# SUNBAY SoftPOS Backend 开发指南

## 目录

1. [开发环境设置](#开发环境设置)
2. [项目架构](#项目架构)
3. [代码规范](#代码规范)
4. [开发工作流](#开发工作流)
5. [测试指南](#测试指南)
6. [调试技巧](#调试技巧)
7. [性能优化](#性能优化)
8. [安全最佳实践](#安全最佳实践)

---

## 开发环境设置

### 必需工具

1. **Rust** (1.75+)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```

2. **SQLite** (3.40+)
```bash
# macOS
brew install sqlite

# Ubuntu/Debian
sudo apt-get install sqlite3 libsqlite3-dev

# Windows
# 从 https://www.sqlite.org/download.html 下载
```

3. **Redis** (7+，可选)
```bash
# macOS
brew install redis
brew services start redis

# Ubuntu/Debian
sudo apt-get install redis-server
sudo systemctl start redis


```

### 推荐工具

- **rust-analyzer**: IDE智能提示
- **cargo-watch**: 自动重新编译
- **cargo-audit**: 安全审计
- **cargo-tarpaulin**: 代码覆盖率

```bash
# 安装推荐工具
cargo install cargo-watch cargo-audit cargo-tarpaulin
```

### IDE配置

#### VS Code

推荐扩展：
- rust-analyzer
- CodeLLDB (调试)
- Better TOML
- Error Lens

`.vscode/settings.json`:
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

---

## 项目架构

### 分层架构

```
┌─────────────────────────────────────┐
│         API Layer (Axum)            │
│  ┌──────────┬──────────┬──────────┐ │
│  │Handlers  │Middleware│ Routes   │ │
│  └──────────┴──────────┴──────────┘ │
└─────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────┐
│       Business Logic Layer          │
│  ┌──────────────────────────────┐   │
│  │ Services (7 core services)   │   │
│  └──────────────────────────────┘   │
└─────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────┐
│      Data Access Layer              │
│  ┌──────────────────────────────┐   │
│  │ Repositories (6 repos)       │   │
│  └──────────────────────────────┘   │
└─────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────┐
│      Infrastructure Layer           │
│  ┌────────┬────────┬──────────────┐ │
│  │Database│ Redis  │ HSM Client   │ │
│  └────────┴────────┴──────────────┘ │
└─────────────────────────────────────┘
```

### 目录结构

```
src/
├── api/                    # API层
│   ├── handlers/          # HTTP处理器
│   │   ├── auth.rs       # 认证处理器
│   │   ├── device.rs     # 设备处理器
│   │   ├── key.rs        # 密钥处理器
│   │   └── ...
│   ├── middleware/        # 中间件
│   │   ├── auth.rs       # 认证中间件
│   │   ├── logging.rs    # 日志中间件
│   │   ├── metrics.rs    # 指标中间件
│   │   └── ...
│   ├── websocket/         # WebSocket
│   │   ├── connection.rs # 连接管理
│   │   └── notification.rs # 通知服务
│   ├── routes.rs          # 路由配置
│   └── mod.rs
├── services/              # 业务逻辑层
│   ├── device.rs         # 设备服务
│   ├── key_management.rs # 密钥管理服务
│   ├── health_check.rs   # 健康检查服务
│   ├── threat_detection.rs # 威胁检测服务
│   ├── transaction.rs    # 交易服务
│   ├── version.rs        # 版本管理服务
│   ├── audit.rs          # 审计服务
│   └── mod.rs
├── repositories/          # 数据访问层
│   ├── device.rs         # 设备仓库
│   ├── health_check.rs   # 健康检查仓库
│   ├── threat.rs         # 威胁仓库
│   ├── transaction.rs    # 交易仓库
│   ├── version.rs        # 版本仓库
│   ├── audit_log.rs      # 审计日志仓库
│   └── mod.rs
├── models/                # 数据模型
│   ├── device.rs         # 设备模型
│   ├── health_check.rs   # 健康检查模型
│   ├── threat.rs         # 威胁模型
│   ├── transaction.rs    # 交易模型
│   ├── version.rs        # 版本模型
│   ├── audit_log.rs      # 审计日志模型
│   └── mod.rs
├── dto/                   # 数据传输对象
│   ├── request.rs        # 请求DTO
│   ├── response.rs       # 响应DTO
│   └── mod.rs
├── security/              # 安全模块
│   ├── jwt.rs            # JWT管理
│   ├── crypto.rs         # 加密工具
│   ├── dukpt.rs          # DUKPT密钥派生
│   └── mod.rs
├── infrastructure/        # 基础设施
│   ├── config.rs         # 配置管理
│   ├── database.rs       # 数据库连接
│   ├── redis.rs          # Redis客户端
│   ├── hsm_client.rs     # HSM客户端
│   └── mod.rs
├── utils/                 # 工具函数
│   ├── error.rs          # 错误处理
│   └── mod.rs
├── lib.rs                 # 库入口
└── main.rs                # 应用入口
```

---

## 代码规范

### Rust代码风格

遵循官方Rust代码风格指南，使用`rustfmt`自动格式化。

#### 命名约定

- **类型名**: PascalCase (`DeviceService`, `HealthCheck`)
- **函数名**: snake_case (`register_device`, `get_device`)
- **常量**: SCREAMING_SNAKE_CASE (`MAX_RETRY_COUNT`)
- **模块名**: snake_case (`key_management`, `health_check`)

#### 示例

```rust
// 好的命名
pub struct DeviceService {
    device_repo: DeviceRepository,
}

impl DeviceService {
    pub async fn register_device(&self, request: RegisterDeviceRequest) -> Result<Device, AppError> {
        // 实现
    }
}

const MAX_DEVICES: usize = 10000;
```

### 错误处理

使用`Result`类型和自定义错误类型：

```rust
use crate::utils::error::AppError;

pub async fn some_function() -> Result<SomeType, AppError> {
    let value = some_operation()
        .map_err(|e| AppError::Database(format!("Failed to query: {}", e)))?;
    
    Ok(value)
}
```

### 异步编程

使用`async/await`和Tokio运行时：

```rust
use tokio::time::{sleep, Duration};

pub async fn async_operation() -> Result<(), AppError> {
    // 异步操作
    let result = tokio::spawn(async {
        sleep(Duration::from_secs(1)).await;
        "done"
    }).await?;
    
    Ok(())
}
```

### 日志记录

使用`tracing`进行结构化日志：

```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(self))]
pub async fn register_device(&self, request: RegisterDeviceRequest) -> Result<Device, AppError> {
    info!(imei = %request.imei, "Registering new device");
    
    // 操作
    
    debug!("Device registered successfully");
    Ok(device)
}
```

### 文档注释

为公共API添加文档注释：

```rust
/// 注册新设备
///
/// # 参数
/// * `request` - 设备注册请求
///
/// # 返回
/// * `Ok(Device)` - 注册成功的设备
/// * `Err(AppError)` - 注册失败的错误
///
/// # 示例
/// ```
/// let request = RegisterDeviceRequest { ... };
/// let device = service.register_device(request).await?;
/// ```
pub async fn register_device(&self, request: RegisterDeviceRequest) -> Result<Device, AppError> {
    // 实现
}
```

---

## 开发工作流

### 1. 创建新功能

```bash
# 创建新分支
git checkout -b feature/new-feature

# 开发
cargo watch -x run

# 测试
cargo test

# 检查
cargo clippy
cargo fmt --check
```

### 2. 添加新的API端点

#### 步骤1: 定义数据模型

```rust
// src/models/example.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Example {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

#### 步骤2: 定义DTO

```rust
// src/dto/request.rs
#[derive(Debug, Deserialize, Validate)]
pub struct CreateExampleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
}

// src/dto/response.rs
#[derive(Debug, Serialize)]
pub struct ExampleResponse {
    pub id: String,
    pub name: String,
    pub created_at: String,
}
```

#### 步骤3: 实现Repository

```rust
// src/repositories/example.rs
pub struct ExampleRepository {
    pool: SqlitePool,
}

impl ExampleRepository {
    pub async fn create(&self, example: &Example) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO examples (id, name, created_at) VALUES (?, ?, ?)",
            example.id,
            example.name,
            example.created_at
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

#### 步骤4: 实现Service

```rust
// src/services/example.rs
pub struct ExampleService {
    example_repo: ExampleRepository,
}

impl ExampleService {
    pub async fn create_example(&self, request: CreateExampleRequest) -> Result<Example, AppError> {
        let example = Example {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            created_at: Utc::now(),
        };
        
        self.example_repo.create(&example).await?;
        
        Ok(example)
    }
}
```

#### 步骤5: 实现Handler

```rust
// src/api/handlers/example.rs
pub async fn create_example(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateExampleRequest>,
) -> Result<Json<ExampleResponse>, AppError> {
    let example = state.example_service.create_example(request).await?;
    
    Ok(Json(ExampleResponse {
        id: example.id,
        name: example.name,
        created_at: example.created_at.to_rfc3339(),
    }))
}
```

#### 步骤6: 注册路由

```rust
// src/api/routes.rs
let protected_routes = Router::new()
    .route("/examples", post(handlers::create_example))
    // ...
```

### 3. 添加数据库迁移

```bash
# 创建迁移文件
touch migrations/20240101000009_create_examples_table.sql
```

```sql
-- migrations/20240101000009_create_examples_table.sql
CREATE TABLE IF NOT EXISTS examples (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_examples_created_at ON examples(created_at);
```

---

## 测试指南

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_creation() {
        let example = Example {
            id: "test-id".to_string(),
            name: "Test".to_string(),
            created_at: Utc::now(),
        };
        
        assert_eq!(example.name, "Test");
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_operation().await;
        assert!(result.is_ok());
    }
}
```

### 集成测试

```rust
// tests/integration_test.rs
use sunbay_softpos_backend::*;

#[tokio::test]
async fn test_device_registration_flow() {
    // 设置测试环境
    let config = Config::from_env().unwrap();
    let state = AppState::new(config).await.unwrap();
    
    // 测试设备注册
    let request = RegisterDeviceRequest {
        imei: "123456789012345".to_string(),
        // ...
    };
    
    let result = state.device_service.register_device(request).await;
    assert!(result.is_ok());
}
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_device_registration

# 运行并显示输出
cargo test -- --nocapture

# 运行被忽略的测试
cargo test -- --ignored

# 生成代码覆盖率报告
cargo tarpaulin --out Html
```

---

## 调试技巧

### 使用日志

```rust
use tracing::{debug, info, warn, error};

debug!("Debug information: {:?}", value);
info!("Operation completed");
warn!("Warning: {}", message);
error!("Error occurred: {}", error);
```

### 使用调试器

#### VS Code配置

`.vscode/launch.json`:
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug",
      "cargo": {
        "args": ["build", "--bin=sunbay-softpos-backend"]
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### 环境变量调试

```bash
# 启用详细日志
RUST_LOG=debug cargo run

# 启用特定模块的日志
RUST_LOG=sunbay_softpos_backend::services=debug cargo run

# 启用回溯
RUST_BACKTRACE=1 cargo run
```

---

## 性能优化

### 数据库优化

1. **使用索引**
```sql
CREATE INDEX idx_devices_status ON devices(status);
CREATE INDEX idx_transactions_device_id ON transactions(device_id);
```

2. **批量操作**
```rust
// 使用事务
let mut tx = pool.begin().await?;
for item in items {
    sqlx::query!("INSERT INTO ...").execute(&mut *tx).await?;
}
tx.commit().await?;
```

3. **连接池配置**
```yaml
database:
  max_connections: 20
  min_connections: 5
  acquire_timeout: 30
```

### 缓存策略

```rust
// 使用Redis缓存
pub async fn get_device_cached(&self, device_id: &str) -> Result<Device, AppError> {
    // 尝试从缓存获取
    if let Some(cached) = self.redis_client.get(device_id).await? {
        return Ok(serde_json::from_str(&cached)?);
    }
    
    // 从数据库获取
    let device = self.device_repo.find_by_id(device_id).await?;
    
    // 写入缓存
    let cached = serde_json::to_string(&device)?;
    self.redis_client.set(device_id, &cached, 3600).await?;
    
    Ok(device)
}
```

### 异步并发

```rust
use tokio::try_join;

// 并发执行多个操作
let (devices, transactions, threats) = try_join!(
    device_repo.list(),
    transaction_repo.list(),
    threat_repo.list()
)?;
```

---

## 安全最佳实践

### 1. 输入验证

```rust
use validator::Validate;

#[derive(Validate)]
pub struct RegisterDeviceRequest {
    #[validate(length(equal = 15))]
    pub imei: String,
    
    #[validate(length(min = 1, max = 100))]
    pub model: String,
}

// 在处理器中验证
request.validate()
    .map_err(|e| AppError::Validation(format!("Invalid input: {}", e)))?;
```

### 2. SQL注入防护

```rust
// 使用参数化查询
sqlx::query!(
    "SELECT * FROM devices WHERE id = ?",
    device_id
)
.fetch_one(&pool)
.await?;
```

### 3. 密码处理

```rust
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

// 哈希密码
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

// 验证密码
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}
```

### 4. JWT安全

```rust
// 使用强密钥
const MIN_SECRET_LENGTH: usize = 32;

// 设置合理的过期时间
const TOKEN_EXPIRATION: i64 = 2 * 3600; // 2小时

// 验证Token
pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
    Ok(token_data.claims)
}
```

### 5. 敏感数据处理

```rust
// 不要在日志中记录敏感信息
info!(device_id = %device_id, "Processing transaction"); // 好
// error!("PIN: {}", pin); // 坏

// 使用环境变量存储敏感配置
let jwt_secret = env::var("JWT_SECRET")
    .expect("JWT_SECRET must be set");
```

---

## 常见问题

### Q: 如何添加新的中间件？

A: 在`src/api/middleware/`中创建新文件，实现中间件函数，然后在`routes.rs`中应用。

### Q: 如何处理数据库迁移失败？

A: 检查迁移文件语法，确保数据库连接正常，必要时手动回滚。

### Q: 如何优化查询性能？

A: 使用索引、限制返回字段、使用缓存、批量操作。

### Q: 如何调试WebSocket连接？

A: 使用浏览器开发者工具或专用WebSocket客户端工具。

---

## 贡献流程

1. Fork项目
2. 创建特性分支
3. 编写代码和测试
4. 运行`cargo fmt`和`cargo clippy`
5. 提交Pull Request
6. 等待代码审查

---

## 资源链接

- [Rust官方文档](https://doc.rust-lang.org/)
- [Axum文档](https://docs.rs/axum/)
- [SQLx文档](https://docs.rs/sqlx/)
- [Tokio文档](https://docs.rs/tokio/)
- [项目设计文档](./design.md)
- [API文档](./API_DOCUMENTATION.md)

---

## 联系方式

如有问题，请通过以下方式联系：

- GitHub Issues
- 技术支持邮箱
- 开发者社区
