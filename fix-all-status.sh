#!/bin/bash

echo "修复所有status比较..."

# 修复所有 device.status != DeviceStatus::XXX
sed -i '' 's/device\.status != DeviceStatus::Pending/device.status != DeviceStatus::Pending.as_str()/g' src/services/device.rs
sed -i '' 's/device\.status != DeviceStatus::Suspended/device.status != DeviceStatus::Suspended.as_str()/g' src/services/device.rs
sed -i '' 's/device\.status == DeviceStatus::Pending/device.status == DeviceStatus::Pending.as_str()/g' src/services/device.rs
sed -i '' 's/device\.status == DeviceStatus::Suspended/device.status == DeviceStatus::Suspended.as_str()/g' src/services/device.rs

echo "完成！"
