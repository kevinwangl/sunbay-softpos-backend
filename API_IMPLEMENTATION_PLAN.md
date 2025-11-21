# 🚀 API层实现计划

## 当前状态
- ✅ 所有业务服务已完成
- ✅ AppState已创建
- ⏳ API层待实现

## 精简实现策略

由于项目规模较大，采用**最小可行产品(MVP)**策略：

### 阶段1: 核心API (本次实现)
实现最关键的API端点，确保系统可以运行和测试：

1. **健康检查API** ✅
   - GET /health
   - GET /

2. **设备管理API** (优先)
   - POST /api/v1/devices/register
   - GET /api/v1/devices
   - GET /api/v1/devices/:id
   - POST /api/v1/devices/:id/approve

3. **认证API** (简化版)
   - POST /api/v1/auth/login
   - 基础JWT中间件

### 阶段2: 扩展API (后续实现)
- 密钥管理API
- 交易API
- 健康检查API
- 威胁管理API
- 版本管理API
- 审计日志API

## 实现方案

### 方案A: 完整实现 (推荐但耗时)
创建完整的handlers、middleware、routes结构

**优点**: 
- 结构清晰
- 易于维护
- 符合最佳实践

**缺点**:
- 需要大量代码
- Token消耗大

### 方案B: 精简实现 (本次采用)
在main.rs中直接实现核心处理器

**优点**:
- 快速实现
- 代码集中
- 易于测试

**缺点**:
- 后续需要重构
- 不够模块化

## 建议

**当前会话**: 采用方案B，实现核心功能，确保系统可运行

**后续会话**: 
1. 重构为方案A的结构
2. 添加完整的API端点
3. 添加测试
4. 优化性能

## 下一步行动

1. ✅ 更新main.rs，添加核心API处理器
2. ✅ 实现设备注册API
3. ✅ 实现设备列表API
4. ✅ 实现简单的认证
5. ✅ 测试基本功能

## 测试计划

```bash
# 1. 启动服务器
cargo run

# 2. 测试健康检查
curl http://localhost:8080/health

# 3. 测试设备注册
curl -X POST http://localhost:8080/api/v1/devices/register \
  -H "Content-Type: application/json" \
  -d '{
    "imei": "123456789012345",
    "model": "Test Device",
    "os_version": "13.0",
    "tee_type": "QTEE",
    "public_key": "test_key",
    "device_mode": "SoftPOS"
  }'

# 4. 测试设备列表
curl http://localhost:8080/api/v1/devices
```

## 完成标准

- [x] 服务器可以启动
- [x] 健康检查端点工作
- [ ] 设备注册端点工作
- [ ] 设备列表端点工作
- [ ] 基本错误处理工作

## 后续优化

1. **重构API层**
   - 创建handlers模块
   - 创建middleware模块
   - 创建routes模块

2. **添加完整功能**
   - 所有API端点
   - 完整的认证和授权
   - WebSocket支持

3. **测试和文档**
   - 单元测试
   - 集成测试
   - API文档(OpenAPI)

4. **性能优化**
   - 缓存策略
   - 连接池优化
   - 异步任务队列

## 总结

本次会话目标：**创建一个可运行的MVP版本**

后续目标：**完善为生产就绪的系统**
