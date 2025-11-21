use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use ring::signature::{self, UnparsedPublicKey, RSA_PKCS1_2048_8192_SHA256};
use crate::utils::error::AppError;

/// 密码哈希
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::InternalWithMessage(format!("Failed to hash password: {}", e)))?
        .to_string();

    Ok(password_hash)
}

/// 验证密码
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| AppError::InternalWithMessage(format!("Failed to parse password hash: {}", e)))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// RSA公钥加密（使用设备公钥加密数据）
pub fn encrypt_with_public_key(_public_key_pem: &str, data: &[u8]) -> Result<Vec<u8>, AppError> {
    // 注意：这是一个简化实现
    // 在生产环境中，应该使用完整的RSA加密实现
    // 这里我们只是返回一个占位符
    
    // 实际实现需要：
    // 1. 解析PEM格式的公钥
    // 2. 使用公钥加密数据
    // 3. 返回加密后的数据
    
    tracing::warn!("RSA encryption is a placeholder implementation");
    
    // 占位符：返回base64编码的数据
    Ok(base64::encode(data).into_bytes())
}

/// 验证RSA签名
pub fn verify_signature(
    public_key_der: &[u8],
    message: &[u8],
    signature: &[u8],
) -> Result<bool, AppError> {
    let public_key = UnparsedPublicKey::new(&RSA_PKCS1_2048_8192_SHA256, public_key_der);

    match public_key.verify(message, signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// 生成随机字节
pub fn generate_random_bytes(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

/// 生成随机十六进制字符串
pub fn generate_random_hex(length: usize) -> String {
    let bytes = generate_random_bytes(length);
    hex::encode(bytes)
}

/// Base64编码
pub fn base64_encode(data: &[u8]) -> String {
    base64::encode(data)
}

/// Base64解码
pub fn base64_decode(data: &str) -> Result<Vec<u8>, AppError> {
    base64::decode(data)
        .map_err(|e| AppError::BadRequest(format!("Failed to decode base64: {}", e)))
}

/// 计算SHA256哈希
pub fn sha256_hash(data: &[u8]) -> Vec<u8> {
    use ring::digest;
    let hash = digest::digest(&digest::SHA256, data);
    hash.as_ref().to_vec()
}

/// 计算SHA256哈希并返回十六进制字符串
pub fn sha256_hash_hex(data: &[u8]) -> String {
    hex::encode(sha256_hash(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_random_generation() {
        let bytes1 = generate_random_bytes(16);
        let bytes2 = generate_random_bytes(16);

        assert_eq!(bytes1.len(), 16);
        assert_eq!(bytes2.len(), 16);
        assert_ne!(bytes1, bytes2);
    }

    #[test]
    fn test_random_hex() {
        let hex1 = generate_random_hex(16);
        let hex2 = generate_random_hex(16);

        assert_eq!(hex1.len(), 32); // 16 bytes = 32 hex chars
        assert_ne!(hex1, hex2);
    }

    #[test]
    fn test_base64() {
        let data = b"Hello, World!";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();

        assert_eq!(data.to_vec(), decoded);
    }

    #[test]
    fn test_sha256() {
        let data = b"Hello, World!";
        let hash = sha256_hash(data);
        let hash_hex = sha256_hash_hex(data);

        assert_eq!(hash.len(), 32); // SHA256 produces 32 bytes
        assert_eq!(hash_hex.len(), 64); // 32 bytes = 64 hex chars
    }
}
