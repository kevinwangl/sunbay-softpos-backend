# Redis 配置指南

## 概述

后端支持多种 Redis 配置方式，包括无认证、密码认证和用户名+密码认证（Redis 6.0+）。

## 配置方式

### 方式 1: 环境变量（推荐用于生产环境）

在 `.env` 文件或系统环境变量中配置：

```bash
# 基本 URL
APP_REDIS__URL=redis://localhost:6379

# 可选：用户名（Redis 6.0+ ACL）
APP_REDIS__USERNAME=default

# 可选：密码
APP_REDIS__PASSWORD=your-secure-password
```

### 方式 2: YAML 配置文件

在 `config/production.yaml` 或 `config/development.yaml` 中配置：

```yaml
redis:
  url: "redis://localhost:6379"
  username: "default"  # 可选
  password: "your-password"  # 可选
```

### 方式 3: URL 内嵌认证（不推荐）

直接在 URL 中包含认证信息：

```bash
# 仅密码
APP_REDIS__URL=redis://:password@localhost:6379

# 用户名和密码
APP_REDIS__URL=redis://username:password@localhost:6379/0
```

**注意**: 此方式不推荐，因为密码会出现在日志中（虽然会被屏蔽）。

## 配置优先级

配置的优先级从高到低：

1. 环境变量 (`APP_REDIS__*`)
2. YAML 配置文件
3. URL 内嵌认证

如果同时配置了多种方式，系统会自动合并，环境变量优先。

## Redis 版本支持

### Redis 5.x 及更早版本

仅支持密码认证：

```bash
APP_REDIS__URL=redis://localhost:6379
APP_REDIS__PASSWORD=your-password
```

或

```bash
APP_REDIS__URL=redis://:your-password@localhost:6379
```

### Redis 6.0+

支持 ACL（访问控制列表），可以使用用户名+密码：

```bash
APP_REDIS__URL=redis://localhost:6379
APP_REDIS__USERNAME=myuser
APP_REDIS__PASSWORD=mypassword
```

## 配置示例

### 示例 1: 无认证（开发环境）

```bash
APP_REDIS__URL=redis://localhost:6379
```

### 示例 2: 仅密码认证

```bash
APP_REDIS__URL=redis://localhost:6379
APP_REDIS__PASSWORD=my-secure-password
```

### 示例 3: 用户名+密码认证（Redis 6.0+）

```bash
APP_REDIS__URL=redis://localhost:6379
APP_REDIS__USERNAME=admin
APP_REDIS__PASSWORD=admin-password
```

### 示例 4: Redis Sentinel

```bash
APP_REDIS__URL=redis://sentinel1:26379,sentinel2:26379,sentinel3:26379
APP_REDIS__PASSWORD=password
```

### 示例 5: Redis Cluster

```bash
APP_REDIS__URL=redis://node1:6379,node2:6379,node3:6379
APP_REDIS__PASSWORD=cluster-password
```

### 示例 6: TLS/SSL 连接

```bash
APP_REDIS__URL=rediss://localhost:6380
APP_REDIS__PASSWORD=secure-password
```

注意：使用 `rediss://` 协议（双 s）表示 TLS 连接。

## 安全最佳实践

### 1. 使用环境变量

**不要**在配置文件中硬编码密码：

```yaml
# ❌ 不推荐
redis:
  password: "hardcoded-password"
```

**应该**使用环境变量：

```yaml
# ✅ 推荐
redis:
  url: "redis://localhost:6379"
  # password 通过环境变量 APP_REDIS__PASSWORD 配置
```

### 2. 使用强密码

```bash
# 生成强密码
openssl rand -base64 32
```

### 3. 限制 Redis 访问

在 `redis.conf` 中：

```conf
# 绑定到本地接口
bind 127.0.0.1

# 启用密码认证
requirepass your-strong-password

# Redis 6.0+: 使用 ACL
user default on >your-strong-password ~* &* +@all
user readonly on >readonly-password ~* &* +@read
```

### 4. 使用 TLS

对于生产环境，建议启用 TLS：

```bash
APP_REDIS__URL=rediss://redis.example.com:6380
APP_REDIS__PASSWORD=secure-password
```

## 故障排查

### 问题 1: 连接被拒绝

```
Error: Connection refused (os error 111)
```

**解决方案**:
- 检查 Redis 服务是否运行：`redis-cli ping`
- 检查 Redis 绑定地址：`redis-cli CONFIG GET bind`
- 检查防火墙设置

### 问题 2: 认证失败

```
Error: NOAUTH Authentication required
```

**解决方案**:
- 确认 Redis 配置了密码：`redis-cli CONFIG GET requirepass`
- 检查配置的密码是否正确
- 测试连接：`redis-cli -a your-password ping`

### 问题 3: 用户名不存在

```
Error: WRONGPASS invalid username-password pair
```

**解决方案**:
- 确认 Redis 版本 >= 6.0
- 检查用户是否存在：`redis-cli ACL LIST`
- 创建用户：`redis-cli ACL SETUSER myuser on >password ~* +@all`

### 问题 4: 权限不足

```
Error: NOPERM this user has no permissions to access one of the keys
```

**解决方案**:
- 检查用户权限：`redis-cli ACL GETUSER username`
- 授予必要权限：`redis-cli ACL SETUSER username +@all`

## 测试连接

### 使用 redis-cli

```bash
# 无认证
redis-cli -h localhost -p 6379 ping

# 仅密码
redis-cli -h localhost -p 6379 -a password ping

# 用户名+密码（Redis 6.0+）
redis-cli -h localhost -p 6379 --user username --pass password ping
```

### 使用后端健康检查

```bash
curl http://localhost:8080/health
```

响应应包含 Redis 状态：

```json
{
  "status": "healthy",
  "redis": "connected",
  "database": "connected"
}
```

## 日志

Redis 连接信息会记录在日志中，敏感信息会被自动屏蔽：

```
INFO Initializing Redis client
DEBUG Redis URL: redis://***@localhost:6379
INFO Redis client initialized successfully
```

## 性能优化

### 连接池配置

当前使用 `ConnectionManager`，自动管理连接池。

### 超时配置

可以在 URL 中配置超时：

```bash
APP_REDIS__URL=redis://localhost:6379?timeout=5s&connect_timeout=3s
```

### 持久化连接

`ConnectionManager` 会自动维护持久化连接，无需额外配置。

## 监控

### 查看 Redis 信息

```bash
redis-cli INFO
```

### 监控连接数

```bash
redis-cli CLIENT LIST
```

### 查看慢查询

```bash
redis-cli SLOWLOG GET 10
```

## 更多资源

- [Redis 官方文档](https://redis.io/documentation)
- [Redis ACL 文档](https://redis.io/topics/acl)
- [redis-rs 文档](https://docs.rs/redis/)
