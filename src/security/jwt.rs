use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use crate::utils::error::AppError;
use crate::models::TransactionTokenClaims;

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (user ID)
    pub username: String, // Username
    pub role: String,     // User role
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
}

/// JWT Service
#[derive(Clone)]
pub struct JwtService {
    secret: String,
    access_token_expiry: i64,  // seconds
    refresh_token_expiry: i64, // seconds
}

impl JwtService {
    /// 创建新的JwtService
    pub fn new(secret: String, expiration: i64) -> Self {
        Self {
            secret,
            access_token_expiry: expiration,
            refresh_token_expiry: expiration * 7, // Refresh token有效期为access token的7倍
        }
    }

    /// 生成Access Token
    pub fn generate_token(
        &self,
        user_id: &str,
        username: &str,
        role: &str,
    ) -> Result<String, AppError> {
        let now = chrono::Utc::now().timestamp();
        let exp = now + self.access_token_expiry;

        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            role: role.to_string(),
            exp,
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::InternalWithMessage(format!("Failed to generate token: {}", e)))?;

        Ok(token)
    }

    /// 生成Refresh Token
    pub fn generate_refresh_token(
        &self,
        user_id: &str,
        username: &str,
        role: &str,
    ) -> Result<String, AppError> {
        let now = chrono::Utc::now().timestamp();
        let exp = now + self.refresh_token_expiry;

        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            role: role.to_string(),
            exp,
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::InternalWithMessage(format!("Failed to generate refresh token: {}", e)))?;

        Ok(token)
    }

    /// 验证Token
    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims)
    }

    /// 刷新Token
    pub fn refresh_token(&self, refresh_token: &str) -> Result<(String, String), AppError> {
        // 验证refresh token
        let claims = self.verify_token(refresh_token)?;

        // 生成新的access token和refresh token
        let access_token = self.generate_token(&claims.sub, &claims.username, &claims.role)?;
        let new_refresh_token =
            self.generate_refresh_token(&claims.sub, &claims.username, &claims.role)?;

        Ok((access_token, new_refresh_token))
    }

    /// 从Token中提取用户ID
    pub fn extract_user_id(&self, token: &str) -> Result<String, AppError> {
        let claims = self.verify_token(token)?;
        Ok(claims.sub)
    }

    /// 从Token中提取用户名
    pub fn extract_username(&self, token: &str) -> Result<String, AppError> {
        let claims = self.verify_token(token)?;
        Ok(claims.username)
    }

    /// 从Token中提取角色
    pub fn extract_role(&self, token: &str) -> Result<String, AppError> {
        let claims = self.verify_token(token)?;
        Ok(claims.role)
    }

    /// 生成交易令牌
    pub fn generate_transaction_token(
        &self,
        claims: &TransactionTokenClaims,
    ) -> Result<String, AppError> {
        let token = encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::InternalWithMessage(format!("Failed to generate transaction token: {}", e)))?;

        Ok(token)
    }

    /// 验证交易令牌
    pub fn verify_transaction_token(&self, token: &str) -> Result<TransactionTokenClaims, AppError> {
        let token_data = decode::<TransactionTokenClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Unauthorized(format!("Invalid transaction token: {}", e)))?;

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_token() {
        let service = JwtService::new("test_secret".to_string(), 3600);

        let token = service
            .generate_token("user123", "testuser", "admin")
            .unwrap();

        let claims = service.verify_token(&token).unwrap();

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn test_refresh_token() {
        let service = JwtService::new("test_secret".to_string(), 3600);

        let refresh_token = service
            .generate_refresh_token("user123", "testuser", "admin")
            .unwrap();

        let (new_access_token, new_refresh_token) =
            service.refresh_token(&refresh_token).unwrap();

        assert!(!new_access_token.is_empty());
        assert!(!new_refresh_token.is_empty());
    }

    #[test]
    fn test_extract_user_info() {
        let service = JwtService::new("test_secret".to_string(), 3600);

        let token = service
            .generate_token("user123", "testuser", "admin")
            .unwrap();

        let user_id = service.extract_user_id(&token).unwrap();
        let username = service.extract_username(&token).unwrap();
        let role = service.extract_role(&token).unwrap();

        assert_eq!(user_id, "user123");
        assert_eq!(username, "testuser");
        assert_eq!(role, "admin");
    }
}
