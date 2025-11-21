#!/bin/bash

# 修复Device模型字段名的脚本

echo "修复字段名..."

# 修复 key_injected_at -> ipek_injected_at
find src/services -name "*.rs" -type f -exec sed -i '' 's/device\.key_injected_at/device.ipek_injected_at/g' {} \;

# 修复 device.ksn -> device.current_ksn  
find src/services -name "*.rs" -type f -exec sed -i '' 's/device\.ksn/device.current_ksn/g' {} \;

# 修复 key_usage_count -> key_remaining_count
find src/services -name "*.rs" -type f -exec sed -i '' 's/device\.key_usage_count/device.key_remaining_count/g' {} \;

# 修复 key_updated_at -> updated_at
find src/services -name "*.rs" -type f -exec sed -i '' 's/device\.key_updated_at/device.updated_at/g' {} \;

echo "字段名修复完成！"
