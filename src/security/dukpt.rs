use crate::utils::error::AppError;

/// DUKPT密钥派生服务
/// 
/// 注意：这是一个简化的DUKPT实现，用于演示目的
/// 在生产环境中，应该使用完整的DUKPT标准实现或HSM
#[derive(Clone)]
pub struct DukptKeyDerivation {
    // Base Derivation Key (BDK) - 在实际环境中应该安全存储
    bdk: Vec<u8>,
}

impl DukptKeyDerivation {
    /// 创建新的DUKPT服务
    pub fn new(bdk: Vec<u8>) -> Self {
        Self { bdk }
    }

    /// 派生IPEK (Initial PIN Encryption Key)
    /// 
    /// IPEK = TDES_Encrypt(BDK, KSN[0..7])
    pub fn derive_ipek(&self, ksn: &str) -> Result<Vec<u8>, AppError> {
        // 简化实现：使用SHA256作为派生函数
        // 实际应该使用TDES加密
        
        let ksn_bytes = hex::decode(ksn)
            .map_err(|e| AppError::BadRequest(format!("Invalid KSN format: {}", e)))?;

        if ksn_bytes.len() < 8 {
            return Err(AppError::BadRequest("KSN too short".to_string()));
        }

        // 取KSN的前8字节
        let ksn_part = &ksn_bytes[0..8];

        // 简化的派生：BDK + KSN_PART 的SHA256
        let mut data = self.bdk.clone();
        data.extend_from_slice(ksn_part);

        let ipek = crate::security::crypto::sha256_hash(&data);

        tracing::debug!("Derived IPEK for KSN: {}", ksn);

        Ok(ipek)
    }

    /// 派生Working Key
    /// 
    /// Working Key用于实际的PIN加密
    pub fn derive_working_key(&self, ipek: &[u8], ksn: &str) -> Result<Vec<u8>, AppError> {
        // 简化实现：IPEK + KSN 的SHA256
        // 实际应该使用DUKPT标准算法
        
        let ksn_bytes = hex::decode(ksn)
            .map_err(|e| AppError::BadRequest(format!("Invalid KSN format: {}", e)))?;

        let mut data = ipek.to_vec();
        data.extend_from_slice(&ksn_bytes);

        let working_key = crate::security::crypto::sha256_hash(&data);

        tracing::debug!("Derived Working Key for KSN: {}", ksn);

        Ok(working_key)
    }

    /// 生成初始KSN
    /// 
    /// KSN格式：IIN (5 bytes) + Device ID (5 bytes) + Counter (2 bytes)
    pub fn generate_initial_ksn(&self, device_id: &str) -> Result<String, AppError> {
        // 简化实现：生成20位十六进制KSN
        
        // IIN (Issuer Identification Number) - 5 bytes
        let iin = "FFFF000000"; // 示例IIN

        // Device ID - 取设备ID的前10个字符，不足补0
        let device_part = format!("{:0<10}", &device_id[..device_id.len().min(10)]);

        // Counter - 初始为0
        let counter = "0000";

        let ksn = format!("{}{}{}", iin, device_part, counter);

        Ok(ksn)
    }

    /// 递增KSN
    /// 
    /// 每次使用密钥后，KSN的计数器部分需要递增
    pub fn increment_ksn(&self, current_ksn: &str) -> Result<String, AppError> {
        if current_ksn.len() != 20 {
            return Err(AppError::BadRequest("Invalid KSN length".to_string()));
        }

        // 提取计数器部分（最后4位）
        let prefix = &current_ksn[0..16];
        let counter_str = &current_ksn[16..20];

        // 解析计数器
        let counter = u16::from_str_radix(counter_str, 16)
            .map_err(|e| AppError::BadRequest(format!("Invalid KSN counter: {}", e)))?;

        // 递增
        let new_counter = counter.wrapping_add(1);

        // 格式化新KSN
        let new_ksn = format!("{}{:04X}", prefix, new_counter);

        Ok(new_ksn)
    }

    /// 加密PIN Block (ISO 9564 Format 0)
    /// 
    /// Format 0: 0 + PIN_LENGTH + PIN + PADDING
    pub fn encrypt_pin_block(
        &self,
        pin: &str,
        working_key: &[u8],
    ) -> Result<Vec<u8>, AppError> {
        // 验证PIN
        if pin.len() < 4 || pin.len() > 12 {
            return Err(AppError::BadRequest("PIN must be 4-12 digits".to_string()));
        }

        if !pin.chars().all(|c| c.is_ascii_digit()) {
            return Err(AppError::BadRequest("PIN must contain only digits".to_string()));
        }

        // 构建PIN Block (ISO 9564 Format 0)
        // Format: 0 + PIN_LENGTH + PIN + PADDING (F's)
        let pin_length = format!("{:X}", pin.len());
        let padding = "F".repeat(14 - pin.len());
        let pin_block_str = format!("0{}{}{}", pin_length, pin, padding);

        // 转换为字节
        let pin_block_bytes = hex::decode(&pin_block_str)
            .map_err(|e| AppError::InternalWithMessage(format!("Failed to encode PIN block: {}", e)))?;

        // 简化的加密：XOR with working key
        // 实际应该使用TDES加密
        let mut encrypted = Vec::new();
        for (i, byte) in pin_block_bytes.iter().enumerate() {
            let key_byte = working_key[i % working_key.len()];
            encrypted.push(byte ^ key_byte);
        }

        tracing::debug!("Encrypted PIN block");

        Ok(encrypted)
    }

    /// 解密PIN Block
    pub fn decrypt_pin_block(
        &self,
        encrypted_pin_block: &[u8],
        working_key: &[u8],
    ) -> Result<String, AppError> {
        // 简化的解密：XOR with working key
        let mut decrypted = Vec::new();
        for (i, byte) in encrypted_pin_block.iter().enumerate() {
            let key_byte = working_key[i % working_key.len()];
            decrypted.push(byte ^ key_byte);
        }

        // 解析PIN Block
        let pin_block_hex = hex::encode(&decrypted);

        // 验证格式
        if !pin_block_hex.starts_with('0') {
            return Err(AppError::BadRequest("Invalid PIN block format".to_string()));
        }

        // 提取PIN长度
        let pin_length = u8::from_str_radix(&pin_block_hex[1..2], 16)
            .map_err(|e| AppError::BadRequest(format!("Invalid PIN length: {}", e)))?;

        // 提取PIN
        let pin = &pin_block_hex[2..(2 + pin_length as usize)];

        Ok(pin.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_service() -> DukptKeyDerivation {
        let bdk = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
        DukptKeyDerivation::new(bdk)
    }

    #[test]
    fn test_generate_initial_ksn() {
        let service = create_test_service();
        let ksn = service.generate_initial_ksn("device123").unwrap();

        assert_eq!(ksn.len(), 20);
        assert!(ksn.ends_with("0000")); // Initial counter is 0
    }

    #[test]
    fn test_increment_ksn() {
        let service = create_test_service();
        let ksn = "FFFF000000device1230000";

        let new_ksn = service.increment_ksn(ksn).unwrap();
        assert!(new_ksn.ends_with("0001"));

        let new_ksn2 = service.increment_ksn(&new_ksn).unwrap();
        assert!(new_ksn2.ends_with("0002"));
    }

    #[test]
    fn test_derive_ipek() {
        let service = create_test_service();
        let ksn = "FFFF000000device1230000";

        let ipek = service.derive_ipek(ksn).unwrap();
        assert_eq!(ipek.len(), 32); // SHA256 produces 32 bytes
    }

    #[test]
    fn test_derive_working_key() {
        let service = create_test_service();
        let ksn = "FFFF000000device1230000";

        let ipek = service.derive_ipek(ksn).unwrap();
        let working_key = service.derive_working_key(&ipek, ksn).unwrap();

        assert_eq!(working_key.len(), 32);
    }

    #[test]
    fn test_encrypt_decrypt_pin() {
        let service = create_test_service();
        let pin = "1234";
        let working_key = vec![0x12; 32];

        let encrypted = service.encrypt_pin_block(pin, &working_key).unwrap();
        let decrypted = service.decrypt_pin_block(&encrypted, &working_key).unwrap();

        assert_eq!(pin, decrypted);
    }

    #[test]
    fn test_pin_validation() {
        let service = create_test_service();
        let working_key = vec![0x12; 32];

        // Too short
        assert!(service.encrypt_pin_block("123", &working_key).is_err());

        // Too long
        assert!(service
            .encrypt_pin_block("1234567890123", &working_key)
            .is_err());

        // Non-numeric
        assert!(service.encrypt_pin_block("12a4", &working_key).is_err());

        // Valid
        assert!(service.encrypt_pin_block("1234", &working_key).is_ok());
    }
}
