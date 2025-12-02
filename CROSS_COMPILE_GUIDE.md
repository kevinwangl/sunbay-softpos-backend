# Linux 交叉编译指南

## 概述

本项目支持从 macOS 交叉编译到 Linux x86_64 平台。

## 环境要求

### 已安装的工具

✅ Rust 工具链
✅ musl-cross 工具链（通过 Homebrew 安装）
✅ 交叉编译目标：
  - `x86_64-unknown-linux-gnu`
  - `x86_64-unknown-linux-musl`

### 工具链位置

- musl-gcc: `/opt/homebrew/bin/x86_64-linux-musl-gcc`
- 完整路径: `/opt/homebrew/Cellar/musl-cross/0.9.9_2/libexec/bin/`

## 配置说明

### 1. Cargo 配置 (`.cargo/config.toml`)

```toml
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"
rustflags = ["-C", "target-feature=+crt-static", "-C", "link-arg=-static"]
```

### 2. 依赖配置 (`Cargo.toml`)

关键修改：
- `reqwest` 使用 `rustls-tls` 而非 `native-tls`
- 禁用默认特性以避免 OpenSSL 依赖
- `sqlx` 使用 `runtime-tokio-rustls`

```toml
reqwest = { version = "0.11", features = ["json", "rustls-tls"], default-features = false }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", ...], default-features = false }
```

## 编译方法

### 方法 1: 使用构建脚本（推荐）

```bash
# 编译 musl 静态链接版本（推荐）
./build-linux.sh

# 或指定目标
./build-linux.sh x86_64-unknown-linux-musl
./build-linux.sh x86_64-unknown-linux-gnu
```

### 方法 2: 手动编译

```bash
# 设置环境变量
export SQLX_OFFLINE=true

# 编译 musl 版本（静态链接，无依赖）
cargo build --release --target x86_64-unknown-linux-musl

# 编译 gnu 版本（动态链接 glibc）
cargo build --release --target x86_64-unknown-linux-gnu
```

## 编译产物

### musl 版本（推荐）
- 路径: `target/x86_64-unknown-linux-musl/release/sunbay-softpos-backend`
- 特点: 静态链接，无运行时依赖
- 适用: 任何 Linux x86_64 系统
- 大小: 较大（~30-50MB）

### gnu 版本
- 路径: `target/x86_64-unknown-linux-gnu/release/sunbay-softpos-backend`
- 特点: 动态链接 glibc
- 适用: glibc 2.17+ 的 Linux 系统
- 大小: 较小（~10-20MB）

## 常见问题

### 1. OpenSSL 链接错误

**问题**: `undefined reference to SSL_*`

**解决方案**:
- 确保使用 `rustls-tls` 而非 `native-tls`
- 在 `reqwest` 中禁用默认特性
- 检查所有依赖是否使用 `rustls`

### 2. 编译时间过长

**原因**: 首次编译需要构建所有依赖

**优化**:
```bash
# 使用增量编译
export CARGO_INCREMENTAL=1

# 并行编译
export CARGO_BUILD_JOBS=8
```

### 3. SQLX 编译错误

**解决方案**:
```bash
# 使用离线模式
export SQLX_OFFLINE=true

# 或重新生成 sqlx-data.json
cargo sqlx prepare
```

## 部署到 Linux

### 1. 复制文件

```bash
# 复制二进制文件
scp target/x86_64-unknown-linux-musl/release/sunbay-softpos-backend user@server:/opt/sunbay/

# 复制配置文件
scp -r config user@server:/opt/sunbay/
scp .env.example user@server:/opt/sunbay/.env
```

### 2. 设置权限

```bash
chmod +x /opt/sunbay/sunbay-softpos-backend
```

### 3. 运行

```bash
cd /opt/sunbay
./sunbay-softpos-backend
```

## 验证编译结果

```bash
# 查看二进制信息
file target/x86_64-unknown-linux-musl/release/sunbay-softpos-backend

# 预期输出（musl）:
# ELF 64-bit LSB executable, x86-64, statically linked

# 查看依赖（musl 应该没有动态依赖）
# 注意：在 macOS 上无法直接运行 ldd，需要在 Linux 上验证
```

## 性能对比

| 版本 | 大小 | 启动时间 | 运行时依赖 | 兼容性 |
|------|------|----------|------------|--------|
| musl | 较大 | 略慢 | 无 | 最佳 |
| gnu  | 较小 | 较快 | glibc 2.17+ | 良好 |

## 推荐配置

**生产环境**: 使用 `x86_64-unknown-linux-musl`
- 无依赖，部署简单
- 兼容性最好
- 适合容器化部署

**开发/测试**: 使用 `x86_64-unknown-linux-gnu`
- 编译速度快
- 二进制文件小
- 调试信息完整

## 自动化构建

### GitHub Actions 示例

参考 `.github/workflows/release.yml` 中的配置。

### 本地自动化

```bash
# 创建发布包
./build-linux.sh
cd release/x86_64-unknown-linux-musl
tar czf sunbay-softpos-backend-linux-x86_64.tar.gz *
```

## 故障排查

### 查看详细编译日志

```bash
RUST_LOG=debug cargo build --release --target x86_64-unknown-linux-musl -vv
```

### 清理并重新编译

```bash
cargo clean
rm -rf target/
./build-linux.sh
```

### 检查工具链

```bash
# 检查已安装的目标
rustup target list --installed

# 检查 musl-gcc
which x86_64-linux-musl-gcc
x86_64-linux-musl-gcc --version
```

## 更多资源

- [Rust 交叉编译指南](https://rust-lang.github.io/rustup/cross-compilation.html)
- [musl-cross 文档](https://github.com/richfelker/musl-cross-make)
- [rustls vs OpenSSL](https://github.com/rustls/rustls#why-rustls)
