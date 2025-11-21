#!/bin/bash
set -e

# 创建数据库目录
mkdir -p data

# 设置数据库URL
export DATABASE_URL="sqlite:data/sunbay.db"

# 创建数据库（如果不存在）
sqlite3 data/sunbay.db "SELECT 1;" 2>/dev/null || true

# 运行迁移
echo "Running migrations..."
cargo sqlx migrate run

# 准备离线查询数据
echo "Preparing offline query data..."
cargo sqlx prepare

echo "SQLx offline mode prepared successfully!"
