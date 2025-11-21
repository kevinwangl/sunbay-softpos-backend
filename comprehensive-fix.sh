#!/bin/bash

echo "开始全面修复编译错误..."

# 1. 修复 device.key_max_usage (不存在的字段)
echo "1. 移除 key_max_usage 引用..."
find src/services -name "*.rs" -type f -exec sed -i '' 's/device\.key_max_usage/device.key_total_count/g' {} \;

# 2. 修复 update_key_info 调用 (参数过多)
echo "2. 修复 update_key_info 调用..."
# 这个需要手动修复，因为参数结构变化太大

# 3. 修复 device.device_mode 比较
echo "3. 修复 device_mode 比较..."
find src/services -name "*.rs" -type f -exec sed -i '' 's/device\.device_mode == DeviceMode::/DeviceMode::from_str(\&device.device_mode) == Some(DeviceMode::/g' {} \;
find src/services -name "*.rs" -type f -exec sed -i '' 's/device\.device_mode != DeviceMode::/DeviceMode::from_str(\&device.device_mode) != Some(DeviceMode::/g' {} \;

# 4. 修复 version_number -> version
echo "4. 修复 version_number 字段..."
find src/services -name "*.rs" -type f -exec sed -i '' 's/\.version_number/.version/g' {} \;
find src/services -name "*.rs" -type f -exec sed -i '' 's/version_number:/version:/g' {} \;

# 5. 修复 sdk_version -> os_version
echo "5. 修复 sdk_version 字段..."
find src/services -name "*.rs" -type f -exec sed -i '' 's/\.sdk_version/.os_version/g' {} \;

echo "批量修复完成！需要手动修复的问题："
echo "- update_key_info 调用参数"
echo "- encrypt_with_public_key 的 public_key 参数类型"
echo "- version service 的参数顺序"
echo "- handler 函数签名"
