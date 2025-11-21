#!/bin/bash

# 修复Device status比较问题
# Device.status是String类型，需要与DeviceStatus::Active.as_str()比较

echo "修复device status比较..."

# 修复 device.status != DeviceStatus::Active
find src/services -name "*.rs" -type f -exec sed -i '' 's/device\.status != DeviceStatus::Active/device.status != DeviceStatus::Active.as_str()/g' {} \;

# 修复 device.status == DeviceStatus::Active  
find src/services -name "*.rs" -type f -exec sed -i '' 's/device\.status == DeviceStatus::Active/device.status == DeviceStatus::Active.as_str()/g' {} \;

# 修复 device.device_mode != DeviceMode::SoftPOS
find src/services -name "*.rs" -type f -exec sed -i '' 's/device\.device_mode != DeviceMode::SoftPOS/device.device_mode != DeviceMode::FullPos.as_str()/g' {} \;

# 修复 device.device_mode != DeviceMode::PINPad
find src/services -name "*.rs" -type f -exec sed -i '' 's/device\.device_mode != DeviceMode::PINPad/device.device_mode != DeviceMode::PinPad.as_str()/g' {} \;

echo "device status比较修复完成！"
