#!/bin/bash

# 运行数据库迁移脚本
# Run database migrations

set -e

echo "🔄 Running database migrations..."

# 检查 sqlx-cli 是否安装
if ! command -v sqlx &> /dev/null; then
    echo "❌ sqlx-cli not found. Installing..."
    cargo install sqlx-cli --no-default-features --features sqlite
fi

# 运行迁移
sqlx migrate run

echo "✅ Migrations completed successfully!"
echo ""
echo "📊 To verify, you can check the database:"
echo "   sqlite3 data/sunbay_dev.db '.schema transactions'"
