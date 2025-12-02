#!/bin/bash

# Linux 交叉编译脚本
# Cross-compile script for Linux targets

set -e

echo "🔨 Building Sunbay SoftPOS Backend for Linux..."
echo ""

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 选择目标平台
TARGET="${1:-x86_64-unknown-linux-musl}"

echo -e "${BLUE}Target platform: ${TARGET}${NC}"
echo ""

# 检查目标是否已安装
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo -e "${YELLOW}Installing target ${TARGET}...${NC}"
    rustup target add "$TARGET"
fi

# 设置环境变量
export SQLX_OFFLINE=true

# 清理之前的构建
echo -e "${BLUE}Cleaning previous build...${NC}"
cargo clean --target "$TARGET"

# 开始编译
echo -e "${BLUE}Building release binary for ${TARGET}...${NC}"
echo ""

if [ "$TARGET" = "x86_64-unknown-linux-musl" ]; then
    # musl 静态链接编译
    cargo build --release --target "$TARGET"
elif [ "$TARGET" = "x86_64-unknown-linux-gnu" ]; then
    # glibc 动态链接编译
    cargo build --release --target "$TARGET"
else
    echo -e "${YELLOW}Unknown target: ${TARGET}${NC}"
    exit 1
fi

# 检查编译结果
if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✅ Build successful!${NC}"
    echo ""
    
    BINARY_PATH="target/${TARGET}/release/sunbay-softpos-backend"
    
    if [ -f "$BINARY_PATH" ]; then
        echo -e "${BLUE}Binary information:${NC}"
        ls -lh "$BINARY_PATH"
        echo ""
        
        echo -e "${BLUE}Binary type:${NC}"
        file "$BINARY_PATH"
        echo ""
        
        # 创建发布目录
        RELEASE_DIR="release/${TARGET}"
        mkdir -p "$RELEASE_DIR"
        
        # 复制二进制文件
        cp "$BINARY_PATH" "$RELEASE_DIR/"
        
        # 复制配置文件
        cp -r config "$RELEASE_DIR/"
        cp .env.example "$RELEASE_DIR/.env"
        
        # 创建 README
        cat > "$RELEASE_DIR/README.md" << EOF
# Sunbay SoftPOS Backend - Linux Release

## Target Platform
- Architecture: ${TARGET}
- Build Date: $(date)

## Installation

1. Extract the archive
2. Configure environment variables in \`.env\`
3. Run the binary:
   \`\`\`bash
   ./sunbay-softpos-backend
   \`\`\`

## Configuration

Edit the \`.env\` file and \`config/production.yaml\` to match your environment.

## Requirements

EOF

        if [ "$TARGET" = "x86_64-unknown-linux-musl" ]; then
            echo "- Linux x86_64 (statically linked, no dependencies)" >> "$RELEASE_DIR/README.md"
        else
            echo "- Linux x86_64 with glibc 2.17+" >> "$RELEASE_DIR/README.md"
        fi
        
        echo ""
        echo -e "${GREEN}📦 Release package created in: ${RELEASE_DIR}${NC}"
        echo ""
        echo -e "${BLUE}Package contents:${NC}"
        ls -lh "$RELEASE_DIR"
        
    else
        echo -e "${YELLOW}⚠️  Binary not found at expected location${NC}"
        exit 1
    fi
else
    echo ""
    echo -e "${YELLOW}❌ Build failed${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 Done!${NC}"
