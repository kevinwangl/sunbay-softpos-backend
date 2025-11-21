# SUNBAY SoftPOS A/M-Backend

基于Rust的高性能、高安全性云端鉴证系统，为SoftPOS终端设备提供设备生命周期管理、实时健康监控、威胁检测、密钥管理和交易处理能力。

## 技术栈

- **Rust 1.75+**
- **Axum 0.7** - Web框架
- **SQLx** - 数据库操作（SQLite）
- **Redis** - 缓存
- **Tokio** - 异步运行时
- **JWT** - 认证
- **Argon2** - 密码哈希
- **Tracing** - 日志和追踪

## 项目状态

当前进度：**核心功能完成（~85%）**

✅ 已完成：
- 基础设施层（100%）
- 数据模型层（100%）
- 数据访问层（100%）
- 安全模块（100%）
- 业务逻辑层（100%）
- API层（100%）
- WebSocket通知（100%）
- 监控和可观测性（100%）
- 部署配置（100%）

查看 [BACKEND_PROJECT_COMPLETE.md](./BACKEND_PROJECT_COMPLETE.md) 了解详细进度。

## 快速开始

### 前置要求

- Rust 1.75+
- SQLite 3.40+
- Redis 7+（可选，用于缓存）

### 安装依赖

```bash
# 安装Rust（如果还没有）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone <repository-url>
cd sunbay-softpos-backend

# 构建项目
cargo build
```

### 配置

1. 复制环境变量模板：
```bash
cp .env.example .env
```

2. 编辑 `.env` 文件，设置必要的配置：
```env
RUN_ENV=development
APP_DATABASE__URL=sqlite://data/sunbay_dev.db
APP_REDIS__URL=redis://127.0.0.1:6379
APP_JWT__SECRET=your-secret-key-min-32-chars
```

3. 配置文件位于 `config/` 目录：
- `development.yaml` - 开发环境
- `production.yaml` - 生产环境
- `test.yaml` - 测试环境

### 运行

```bash
# 开发模式
cargo run

# 发布模式
cargo run --release

# 运行测试
cargo test

# 运行特定测试
cargo test infrastructure::config

# 代码检查
cargo clippy

# 格式化代码
cargo fmt
```

### 数据库迁移

数据库迁移会在应用启动时自动执行。迁移文件位于 `migrations/` 目录。

手动运行迁移：
```bash
sqlx migrate run
```

## 项目结构

```
sunbay-softpos-backend/
├── config/                 # 配置文件
├── migrations/             # 数据库迁移
├── src/
│   ├── infrastructure/     # 基础设施层（数据库、Redis、配置）
│   ├── models/            # 数据模型
│   ├── dto/               # 数据传输对象
│   ├── repositories/      # 数据访问层
│   ├── services/          # 业务逻辑层
│   ├── security/          # 安全模块（JWT、加密、DUKPT）
│   ├── api/               # API层（路由、处理器、中间件）
│   ├── utils/             # 工具函数
│   ├── lib.rs             # 库入口
│   └── main.rs            # 应用入口
└── tests/                 # 集成测试
```

## 已实现功能

### ✅ 基础设施层
- 错误处理（统一错误类型和HTTP响应）
- 配置管理（YAML + 环境变量）
- 数据库连接池（SQLite）
- Redis客户端（缓存）
- HSM客户端（FutureX集成）

### ✅ 数据库Schema
- devices（设备表）
- health_checks（健康检查表）
- threat_events（威胁事件表）
- transactions（交易表）
- sdk_versions（SDK版本表）
- audit_logs（审计日志表）
- pin_encryption_logs（PIN加密日志表）

### ✅ 数据模型
- Device（设备模型，支持SoftPOS和PINPad模式）
- HealthCheck（健康检查模型）
- ThreatEvent（威胁事件模型）
- Transaction（交易模型）
- SdkVersion（SDK版本模型）
- AuditLog（审计日志模型）
- 完整的DTO层（30+个请求/响应对象）

### ✅ 安全模块
- JWT Token管理（生成、验证、刷新）
- 加密工具（RSA、签名验证、密码哈希）
- DUKPT密钥派生（IPEK、Working Key、PIN加密）

### ✅ 业务逻辑层
- 设备服务（注册、审批、生命周期管理）
- 健康检查服务（实时监控、安全评分）
- 威胁检测服务（自动检测、严重性评估）
- 密钥管理服务（注入、更新、状态查询）
- 交易服务（鉴证、处理、PIN加密）
- 版本管理服务（创建、分发、更新追踪）
- 审计日志服务（完整操作记录）

### ✅ API层
- 60+ RESTful API端点
- 认证中间件（JWT验证）
- 速率限制中间件（防DDoS）
- 日志中间件（结构化日志）
- 指标中间件（性能监控）
- WebSocket实时通知

### ✅ 监控和可观测性
- 结构化日志（Tracing）
- Prometheus指标导出
- 分布式追踪（Trace ID传播）
- 健康检查端点

### ✅ 部署配置
- Systemd服务文件
- CI/CD配置（GitHub Actions）
- 自动化部署脚本

## 开发指南

### 添加新的API端点

1. 在 `src/models/` 中定义数据模型
2. 在 `src/dto/` 中定义请求/响应DTO
3. 在 `src/repositories/` 中实现数据访问
4. 在 `src/services/` 中实现业务逻辑
5. 在 `src/api/handlers/` 中实现API处理器
6. 在 `src/api/routes.rs` 中注册路由

### 添加数据库迁移

```bash
# 创建新的迁移文件
# 文件名格式：YYYYMMDDHHMMSS_description.sql
touch migrations/20240101000008_add_new_table.sql
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test infrastructure

# 运行并显示输出
cargo test -- --nocapture

# 运行被忽略的测试（需要Redis等外部服务）
cargo test -- --ignored
```

### 代码质量

```bash
# 检查代码
cargo clippy

# 格式化代码
cargo fmt

# 检查安全漏洞
cargo audit
```

## API文档

API遵循RESTful设计，使用JSON格式。

### 认证

使用JWT Bearer Token认证：
```
Authorization: Bearer <token>
```

### 错误响应格式

```json
{
  "error_code": "DEVICE_NOT_FOUND",
  "error_message": "Device not found",
  "details": null
}
```

### 主要端点

#### 认证
- `POST /api/v1/auth/login` - 用户登录
- `POST /api/v1/auth/refresh` - 刷新Token
- `POST /api/v1/auth/logout` - 用户登出
- `GET /api/v1/auth/me` - 获取当前用户信息

#### 设备管理
- `POST /api/v1/devices/register` - 注册设备
- `GET /api/v1/devices` - 查询设备列表
- `GET /api/v1/devices/:id` - 获取设备详情
- `POST /api/v1/devices/:id/approve` - 审批设备
- `POST /api/v1/devices/:id/reject` - 拒绝设备
- `POST /api/v1/devices/:id/suspend` - 暂停设备
- `POST /api/v1/devices/:id/resume` - 恢复设备
- `POST /api/v1/devices/:id/revoke` - 吊销设备
- `GET /api/v1/devices/statistics` - 获取设备统计

#### 密钥管理
- `POST /api/v1/keys/inject` - 密钥注入
- `GET /api/v1/keys/:device_id/status` - 查询密钥状态
- `POST /api/v1/keys/:device_id/update` - 更新密钥

#### 健康检查
- `POST /api/v1/health-checks` - 提交健康检查
- `GET /api/v1/health-checks` - 查询健康检查记录
- `GET /api/v1/health-checks/overview` - 获取健康概览

#### 威胁管理
- `GET /api/v1/threats` - 查询威胁列表
- `GET /api/v1/threats/:id` - 获取威胁详情
- `POST /api/v1/threats/:id/resolve` - 解决威胁
- `GET /api/v1/threats/statistics` - 获取威胁统计

#### 交易处理
- `POST /api/v1/transactions/attest` - 交易鉴证
- `POST /api/v1/transactions/process` - 处理交易
- `GET /api/v1/transactions` - 查询交易记录
- `GET /api/v1/transactions/:id` - 获取交易详情

#### PINPad模式
- `POST /api/v1/pinpad/attest` - PINPad设备鉴证
- `POST /api/v1/pinpad/encrypt-pin` - PIN加密
- `GET /api/v1/pinpad/logs` - 查询PIN加密日志

#### 版本管理
- `POST /api/v1/versions` - 创建版本
- `GET /api/v1/versions` - 查询版本列表
- `GET /api/v1/versions/:id` - 获取版本详情
- `PUT /api/v1/versions/:id` - 更新版本
- `GET /api/v1/versions/statistics` - 获取版本统计
- `POST /api/v1/versions/push` - 推送版本更新

#### 审计日志
- `GET /api/v1/audit-logs` - 查询审计日志
- `GET /api/v1/audit-logs/:id` - 获取日志详情

#### 系统
- `GET /health/check` - 健康检查
- `GET /metrics` - Prometheus指标
- `GET /ws` - WebSocket连接

详细API文档请参考 [API_DOCUMENTATION.md](./API_DOCUMENTATION.md)。

## 配置说明

### 服务器配置
```yaml
server:
  host: "0.0.0.0"
  port: 8080
```

### 数据库配置
```yaml
database:
  url: "sqlite://data/sunbay.db"
  max_connections: 10
```

### Redis配置
```yaml
redis:
  url: "redis://localhost:6379"
```

### JWT配置
```yaml
jwt:
  secret: "your-secret-key"  # 至少32字符
  expiration_hours: 2
  refresh_expiration_days: 7
```

### HSM配置
```yaml
hsm:
  base_url: "https://hsm.futurex.com"
  api_key: "your-api-key"
  timeout_seconds: 30
```

## 部署

### 使用Systemd

1. 构建发布版本：
```bash
cargo build --release
```

2. 复制二进制文件：
```bash
sudo cp target/release/sunbay-softpos-backend /opt/sunbay/
```

3. 创建systemd服务文件（参考 `deploy/sunbay-softpos-backend.service`）

4. 启动服务：
```bash
sudo systemctl enable sunbay-softpos-backend
sudo systemctl start sunbay-softpos-backend
```



## 监控

### 健康检查

```bash
curl http://localhost:8080/health
```

### Prometheus指标

```bash
curl http://localhost:8080/metrics
```

## 安全注意事项

1. **JWT密钥**：生产环境必须使用强密钥（至少32字符）
2. **数据库**：定期备份数据库文件
3. **TLS**：生产环境必须启用TLS 1.3
4. **密钥管理**：敏感配置使用环境变量，不要提交到版本控制
5. **审计日志**：定期审查审计日志

## 故障排查

### 数据库连接失败
- 检查数据库文件路径
- 确认目录权限
- 查看日志：`journalctl -u sunbay-softpos-backend`

### Redis连接失败
- 确认Redis服务运行：`redis-cli ping`
- 检查Redis URL配置
- 系统可以在没有Redis的情况下运行（性能会降低）

### HSM调用失败
- 检查HSM URL和API密钥
- 确认网络连接
- 查看HSM错误日志

## 贡献指南

1. Fork项目
2. 创建特性分支：`git checkout -b feature/new-feature`
3. 提交更改：`git commit -am 'Add new feature'`
4. 推送分支：`git push origin feature/new-feature`
5. 提交Pull Request

## 许可证

[待定]

## 联系方式

- 项目负责人：[待定]
- 技术支持：[待定]
- 问题反馈：[GitHub Issues]

## 更新日志

### v0.1.0 (2024-01-01)
- 初始版本
- 基础设施层完成
- 数据库Schema设计完成
- 核心数据模型完成
