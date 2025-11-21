#!/bin/bash

# SUNBAY SoftPOS Backend 部署脚本
# 用法: ./deploy.sh [environment]
# environment: development, staging, production (默认: production)

set -e

ENVIRONMENT=${1:-production}
APP_NAME="sunbay-softpos-backend"
INSTALL_DIR="/opt/sunbay-softpos"
CONFIG_DIR="/etc/sunbay-softpos"
DATA_DIR="/var/lib/sunbay-softpos"
LOG_DIR="/var/log/sunbay-softpos"
SERVICE_FILE="sunbay-softpos.service"
USER="sunbay"
GROUP="sunbay"

echo "========================================="
echo "SUNBAY SoftPOS Backend 部署脚本"
echo "环境: $ENVIRONMENT"
echo "========================================="

# 检查是否以root权限运行
if [ "$EUID" -ne 0 ]; then
    echo "错误: 请使用root权限运行此脚本"
    exit 1
fi

# 1. 创建用户和组
echo "步骤 1/10: 创建用户和组..."
if ! id "$USER" &>/dev/null; then
    useradd -r -s /bin/false -d "$INSTALL_DIR" "$USER"
    echo "用户 $USER 已创建"
else
    echo "用户 $USER 已存在"
fi

# 2. 创建目录
echo "步骤 2/10: 创建目录..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$CONFIG_DIR"
mkdir -p "$DATA_DIR"
mkdir -p "$LOG_DIR"

# 3. 构建应用
echo "步骤 3/10: 构建应用..."
cargo build --release

# 4. 复制二进制文件
echo "步骤 4/10: 复制二进制文件..."
cp target/release/$APP_NAME "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/$APP_NAME"

# 5. 复制配置文件
echo "步骤 5/10: 复制配置文件..."
if [ -f "config/$ENVIRONMENT.yaml" ]; then
    cp "config/$ENVIRONMENT.yaml" "$CONFIG_DIR/config.yaml"
else
    echo "警告: 配置文件 config/$ENVIRONMENT.yaml 不存在"
fi

# 6. 复制环境变量文件
echo "步骤 6/10: 复制环境变量文件..."
if [ -f ".env.$ENVIRONMENT" ]; then
    cp ".env.$ENVIRONMENT" "$CONFIG_DIR/.env"
else
    echo "警告: 环境变量文件 .env.$ENVIRONMENT 不存在"
fi

# 7. 设置权限
echo "步骤 7/10: 设置权限..."
chown -R "$USER:$GROUP" "$INSTALL_DIR"
chown -R "$USER:$GROUP" "$CONFIG_DIR"
chown -R "$USER:$GROUP" "$DATA_DIR"
chown -R "$USER:$GROUP" "$LOG_DIR"

# 8. 安装systemd服务
echo "步骤 8/10: 安装systemd服务..."
cp "$SERVICE_FILE" /etc/systemd/system/
systemctl daemon-reload

# 9. 启用并启动服务
echo "步骤 9/10: 启用并启动服务..."
systemctl enable $APP_NAME
systemctl restart $APP_NAME

# 10. 检查服务状态
echo "步骤 10/10: 检查服务状态..."
sleep 2
systemctl status $APP_NAME --no-pager

echo ""
echo "========================================="
echo "部署完成!"
echo "========================================="
echo "服务名称: $APP_NAME"
echo "安装目录: $INSTALL_DIR"
echo "配置目录: $CONFIG_DIR"
echo "数据目录: $DATA_DIR"
echo "日志目录: $LOG_DIR"
echo ""
echo "常用命令:"
echo "  查看状态: systemctl status $APP_NAME"
echo "  查看日志: journalctl -u $APP_NAME -f"
echo "  重启服务: systemctl restart $APP_NAME"
echo "  停止服务: systemctl stop $APP_NAME"
echo "========================================="
