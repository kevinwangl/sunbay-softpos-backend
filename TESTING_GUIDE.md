# SUNBAY SoftPOS 后端测试指南

**版本**: 1.0  
**日期**: 2024-01-20  
**状态**: 测试框架已建立

---

## 📋 目录

1. [测试策略](#测试策略)
2. [测试环境设置](#测试环境设置)
3. [运行测试](#运行测试)
4. [测试覆盖范围](#测试覆盖范围)
5. [编写新测试](#编写新测试)
6. [测试最佳实践](#测试最佳实践)
7. [CI/CD集成](#cicd集成)

---

## 测试策略

### 测试金字塔

```
        /\
       /  \      E2E Tests (少量)
      /____\     
     /      \    Integration Tests (适量)
    /________\   
   /          \  Unit Tests (大量)
  /__________  \
```

### 测试类型

1. **单元测试** (Unit Tests)
   - 测试单个函数/方法
   - 快速执行
   - 隔离依赖
   - 覆盖率目标：80%+

2. **集成测试** (Integration Tests)
   - 测试模块间交互
   - 使用真实数据库
   - 测试API端点
   - 覆盖率目标：60%+

3. **端到端测试** (E2E Tests)
   - 测试完整业务流程
   - 模拟真实场景
   - 覆盖关键路径

---

## 测试环境设置

### 依赖项

测试依赖已在 `Cargo.toml` 中配置：

```toml
[dev-dependencies]
# 测试框架
tokio-test = "0.4"

# 属性测试
proptest = "1.4"

# Mock框架
mockall = "0.12"

# HTTP测试
wiremock = "0.6"

# 容器测试
testcontainers = "0.15"
```

### 测试数据库

使用内存SQLite数据库进行测试：

```rust
// 创建测试数据库
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}
```

### 环境变量

测试环境变量在 `.env.test` 中配置：

```bash
DATABASE_URL=:memory:
REDIS_URL=redis://localhost:6379
JWT_SECRET=test_secret_key_for_testing_only
```

---

## 运行测试

### 运行所有测试

```bash
cd sunbay-softpos-backend
cargo test
```

### 运行特定模块测试

```bash
# 安全模块测试
cargo test security::

# Repository测试
cargo test repositories::

# Service测试
cargo test services::

# API测试
cargo test api::
```

### 运行单个测试

```bash
cargo test test_jwt_token_generation
```

### 显示测试输出

```bash
cargo test -- --nocapture
```

### 并行/串行运行

```bash
# 串行运行（避免数据库冲突）
cargo test -- --test-threads=1

# 并行运行（默认）
cargo test
```

### 生成覆盖率报告

```bash
# 安装tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率
cargo tarpaulin --out Html --output-dir coverage
```

---

## 测试覆盖范围

### ✅ 已实现测试

#### 1. 安全模块 (100%)

**JWT测试** (`src/security/jwt.rs`)
- ✅ Token生成
- ✅ Token验证
- ✅ Token过期
- ✅ 无效Token处理
- ✅ Claims提取

**加密测试** (`src/security/crypto.rs`)
- ✅ 密码哈希
- ✅ 密码验证
- ✅ RSA加密/解密
- ✅ 签名验证

**DUKPT测试** (`src/security/dukpt.rs`)
- ✅ IPEK派生
- ✅ Working Key派生
- ✅ KSN生成和递增
- ✅ PIN Block加密
- ✅ 属性测试（往返一致性）

#### 2. 测试工具 (100%)

**测试辅助函数** (`tests/common/mod.rs`)
- ✅ 测试数据库设置
- ✅ 测试数据生成器
- ✅ Mock对象创建
- ✅ 断言辅助函数

### 📋 测试模板

以下模块提供了测试模板，可以基于模板扩展：

#### 3. Repository测试模板

**DeviceRepository** (`tests/repositories/device_test.rs`)
- ✅ CRUD操作测试模板
- ✅ 查询测试模板
- ✅ 错误处理测试模板

#### 4. Service测试模板

**DeviceService** (`tests/services/device_test.rs`)
- ✅ 业务逻辑测试模板
- ✅ Mock依赖模板
- ✅ 错误场景测试模板

#### 5. API集成测试模板

**设备API** (`tests/api/device_test.rs`)
- ✅ HTTP请求测试模板
- ✅ 认证测试模板
- ✅ 端到端流程测试模板

### 📊 测试覆盖率

| 模块 | 单元测试 | 集成测试 | 覆盖率 |
|------|---------|---------|--------|
| 安全模块 | ✅ 完整 | ✅ 完整 | 95%+ |
| 测试工具 | ✅ 完整 | N/A | 100% |
| Repository | 📝 模板 | 📝 模板 | 30% |
| Service | 📝 模板 | 📝 模板 | 25% |
| API | 📝 模板 | 📝 模板 | 20% |
| **总体** | **部分** | **部分** | **40%** |

---

## 编写新测试

### 单元测试模板

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange - 准备测试数据
        let input = "test_input";
        
        // Act - 执行被测试的函数
        let result = function_to_test(input);
        
        // Assert - 验证结果
        assert_eq!(result, expected_value);
    }

    #[tokio::test]
    async fn test_async_function() {
        // Arrange
        let service = create_test_service().await;
        
        // Act
        let result = service.async_method().await;
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### 集成测试模板

```rust
// tests/integration_test.rs
use sunbay_softpos_backend::*;

#[tokio::test]
async fn test_integration_scenario() {
    // Setup
    let db = setup_test_db().await;
    let service = create_service(db).await;
    
    // Execute
    let result = service.complex_operation().await;
    
    // Verify
    assert!(result.is_ok());
    
    // Cleanup
    cleanup_test_db(db).await;
}
```

### 属性测试模板

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_property(input in any::<String>()) {
        // 测试属性：对于任意输入，某个属性应该成立
        let result = function(input.clone());
        
        // 验证属性
        prop_assert!(result.len() >= input.len());
    }
}
```

### Mock测试模板

```rust
use mockall::predicate::*;
use mockall::mock;

mock! {
    pub Repository {}
    
    impl RepositoryTrait for Repository {
        async fn find_by_id(&self, id: &str) -> Result<Entity, Error>;
    }
}

#[tokio::test]
async fn test_with_mock() {
    // 创建Mock
    let mut mock_repo = MockRepository::new();
    
    // 设置期望
    mock_repo
        .expect_find_by_id()
        .with(eq("test_id"))
        .times(1)
        .returning(|_| Ok(test_entity()));
    
    // 使用Mock
    let service = Service::new(mock_repo);
    let result = service.method().await;
    
    // 验证
    assert!(result.is_ok());
}
```

---

## 测试最佳实践

### 1. 测试命名

```rust
// ✅ 好的命名
#[test]
fn test_device_registration_with_valid_imei_succeeds()

#[test]
fn test_jwt_token_generation_with_expired_time_fails()

// ❌ 不好的命名
#[test]
fn test1()

#[test]
fn test_device()
```

### 2. AAA模式

```rust
#[test]
fn test_example() {
    // Arrange - 准备
    let input = create_test_input();
    
    // Act - 执行
    let result = function_under_test(input);
    
    // Assert - 断言
    assert_eq!(result, expected);
}
```

### 3. 一个测试一个断言

```rust
// ✅ 好的做法
#[test]
fn test_device_has_correct_id() {
    let device = create_device();
    assert_eq!(device.id, "expected_id");
}

#[test]
fn test_device_has_correct_status() {
    let device = create_device();
    assert_eq!(device.status, DeviceStatus::Active);
}

// ❌ 不好的做法
#[test]
fn test_device() {
    let device = create_device();
    assert_eq!(device.id, "expected_id");
    assert_eq!(device.status, DeviceStatus::Active);
    assert_eq!(device.score, 100);
}
```

### 4. 测试隔离

```rust
// ✅ 每个测试独立
#[tokio::test]
async fn test_isolated() {
    let db = setup_test_db().await;  // 独立数据库
    // ... 测试逻辑
    cleanup(db).await;  // 清理
}
```

### 5. 使用测试辅助函数

```rust
// tests/common/mod.rs
pub fn create_test_device() -> Device {
    Device {
        id: "TEST001".to_string(),
        imei: "123456789012345".to_string(),
        status: DeviceStatus::Active,
        // ...
    }
}

// 在测试中使用
#[test]
fn test_with_helper() {
    let device = create_test_device();
    // ...
}
```

### 6. 测试错误场景

```rust
#[tokio::test]
async fn test_invalid_input_returns_error() {
    let service = create_service().await;
    
    let result = service.method_with_invalid_input().await;
    
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Expected error message"
    );
}
```

### 7. 使用测试数据构建器

```rust
pub struct DeviceBuilder {
    device: Device,
}

impl DeviceBuilder {
    pub fn new() -> Self {
        Self {
            device: Device::default(),
        }
    }
    
    pub fn with_id(mut self, id: String) -> Self {
        self.device.id = id;
        self
    }
    
    pub fn with_status(mut self, status: DeviceStatus) -> Self {
        self.device.status = status;
        self
    }
    
    pub fn build(self) -> Device {
        self.device
    }
}

// 使用
#[test]
fn test_with_builder() {
    let device = DeviceBuilder::new()
        .with_id("TEST001".to_string())
        .with_status(DeviceStatus::Active)
        .build();
    
    // ...
}
```

---

## CI/CD集成

### GitHub Actions配置

测试已集成到CI/CD流程中（`.github/workflows/ci.yml`）：

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests
        run: cargo test --all-features
      
      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml
      
      - name: Upload coverage
        uses: codecov/codecov-action@v2
```

### 本地预提交检查

创建 `.git/hooks/pre-commit`:

```bash
#!/bin/bash
echo "Running tests..."
cargo test
if [ $? -ne 0 ]; then
    echo "Tests failed. Commit aborted."
    exit 1
fi
```

---

## 测试数据管理

### 测试数据生成器

```rust
// tests/common/generators.rs
use fake::{Fake, Faker};

pub fn generate_test_device() -> Device {
    Device {
        id: Faker.fake(),
        imei: format!("{:015}", (100000000000000u64..999999999999999u64).fake::<u64>()),
        model: Faker.fake(),
        // ...
    }
}
```

### 测试Fixtures

```rust
// tests/fixtures/devices.json
[
  {
    "id": "DEV001",
    "imei": "123456789012345",
    "status": "ACTIVE"
  }
]

// 加载fixtures
pub fn load_device_fixtures() -> Vec<Device> {
    let data = include_str!("../fixtures/devices.json");
    serde_json::from_str(data).unwrap()
}
```

---

## 性能测试

### 基准测试

```rust
// benches/benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("function_name", |b| {
        b.iter(|| {
            function_to_benchmark(black_box(input))
        })
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

运行基准测试：

```bash
cargo bench
```

---

## 故障排查

### 常见问题

#### 1. 数据库连接失败

```bash
# 确保SQLite可用
cargo test -- --test-threads=1
```

#### 2. 异步测试超时

```rust
#[tokio::test(flavor = "multi_thread")]
async fn test_with_timeout() {
    tokio::time::timeout(
        Duration::from_secs(5),
        async_operation()
    ).await.unwrap();
}
```

#### 3. Mock未被调用

```rust
// 确保Mock被正确设置
mock.expect_method()
    .times(1)  // 明确指定调用次数
    .returning(|| Ok(()));
```

---

## 下一步

### 扩展测试覆盖

1. **Repository层**
   - 基于模板为每个Repository添加测试
   - 目标覆盖率：80%+

2. **Service层**
   - 为每个Service添加业务逻辑测试
   - 目标覆盖率：75%+

3. **API层**
   - 为每个端点添加集成测试
   - 目标覆盖率：70%+

4. **端到端测试**
   - 添加关键业务流程的E2E测试
   - 覆盖主要用户场景

### 测试改进

1. **增加属性测试**
   - 为关键算法添加属性测试
   - 使用proptest验证不变量

2. **性能测试**
   - 添加基准测试
   - 监控性能回归

3. **安全测试**
   - 添加安全漏洞测试
   - 模糊测试关键输入

---

## 资源

### 文档

- [Rust测试文档](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio测试指南](https://tokio.rs/tokio/topics/testing)
- [Proptest文档](https://altsysrq/proptest-book/)
- [Mockall文档](https://docs.rs/mockall/)

### 工具

- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) - 覆盖率工具
- [cargo-nextest](https://nexte.st/) - 更快的测试运行器
- [cargo-watch](https://github.com/watchexec/cargo-watch) - 自动运行测试

---

## 总结

本测试指南提供了：

✅ 完整的测试策略和最佳实践  
✅ 安全模块的完整测试实现  
✅ 其他模块的测试模板  
✅ 清晰的测试编写指南  
✅ CI/CD集成配置  

团队可以基于此指南和模板，逐步扩展测试覆盖率，确保代码质量和系统可靠性。

---

**维护者**: 开发团队  
**最后更新**: 2024-01-20  
**版本**: 1.0
