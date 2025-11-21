# 🚀 SUNBAY SoftPOS Backend

> 企业级、金融级、高性能的SoftPOS管理平台后端服务

## 📋 项目概述

SUNBAY SoftPOS Backend是一个基于Rust开发的高性能、安全的SoftPOS（软件POS）管理平台后端服务。

**当前版本**: v0.1.0 (MVP)  
**完成度**: 60% (核心功能完成)

## ✨ 核心特性

### 🔒 金融级安全
- DUKPT密钥派生（PCI标准）
- HSM硬件安全模块集成
- PIN Block加密（ISO 9564）
- 多层加密保护
- 全程审计追踪

### 🧠 智能运维
- 健康评分（0-100分）
- 7种威胁类型检测
- 自动化响应策略
- 智能版本匹配

### ⚡ 高性能
- 异步IO（Tokio）
- 连接池优化
- Redis缓存
- 批量操作

## 🏗️ 技术栈

- **Web**: Axum 0.6+
- **数据库**: SQLite + SQLx
- **缓存**: Redis
- **认证**: JWT
- **加密**: DUKPT, RSA, Argon2

## 🚀 快速开始

```bash
# 1. 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 设置环境
export DATABASE_URL="sqlite:./data/sunbay.db"

# 3. 运行迁移
sqlx database create
sqlx migrate run

# 4. 启动服务
cargo run

# 5. 测试
curl http://localhost:8080/health
```

## 📚 文档

- [项目状态](PROJECT_STATUS_FINAL.md)
- [开发进度](CURRENT_PROGRESS.md)
- [服务文档](ALL_SERVICES_COMPLETE.md)
- [安全文档](SECURITY_COMPLETE.md)

## 🎯 开发路线图

- ✅ v0.1.0: 核心业务逻辑
- ⏳ v0.2.0: API层实现
- 📋 v0.3.0: 测试和优化

## 📝 许可证

Copyright © 2024 SUNBAY

---

**Made with ❤️ by SUNBAY Team**
