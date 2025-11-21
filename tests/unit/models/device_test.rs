// Unit tests for Device model
use crate::models::device::{Device, DeviceStatus};
use chrono::Utc;
use uuid::Uuid;

#[cfg(test)]
mod device_tests {
    use super::*;

    #[test]
    fn test_device_creation() {
        let device = Device {
            id: Uuid::new_v4(),
            device_id: "TEST-DEVICE-001".to_string(),
            device_name: "Test Device".to_string(),
            device_type: "Android".to_string(),
            os_version: "12.0".to_string(),
            app_version: "1.0.0".to_string(),
            status: DeviceStatus::Pending,
            security_score: 85,
            last_heartbeat: Some(Utc::now()),
            registered_at: Utc::now(),
            approved_at: None,
            approved_by: None,
        };

        assert_eq!(device.device_id, "TEST-DEVICE-001");
        assert_eq!(device.status, DeviceStatus::Pending);
        assert_eq!(device.security_score, 85);
    }

    #[test]
    fn test_device_status_transitions() {
        let mut device = Device {
            id: Uuid::new_v4(),
            device_id: "TEST-DEVICE-002".to_string(),
            device_name: "Test Device 2".to_string(),
            device_type: "iOS".to_string(),
            os_version: "15.0".to_string(),
            app_version: "1.0.0".to_string(),
            status: DeviceStatus::Pending,
            security_score: 90,
            last_heartbeat: None,
            registered_at: Utc::now(),
            approved_at: None,
            approved_by: None,
        };

        // Test status transition from Pending to Active
        device.status = DeviceStatus::Active;
        assert_eq!(device.status, DeviceStatus::Active);

        // Test status transition to Suspended
        device.status = DeviceStatus::Suspended;
        assert_eq!(device.status, DeviceStatus::Suspended);
    }

    #[test]
    fn test_security_score_validation() {
        // Valid security scores
        assert!(validate_security_score(0));
        assert!(validate_security_score(50));
        assert!(validate_security_score(100));

        // Invalid security scores
        assert!(!validate_security_score(-1));
        assert!(!validate_security_score(101));
    }

    #[test]
    fn test_device_id_format() {
        let valid_ids = vec![
            "DEVICE-001",
            "TEST-DEVICE-123",
            "PROD-DEVICE-999",
        ];

        for id in valid_ids {
            assert!(is_valid_device_id(id), "Device ID {} should be valid", id);
        }

        let invalid_ids = vec![
            "",
            "DEVICE",
            "device-001",  // lowercase
            "DEVICE_001",  // underscore instead of hyphen
        ];

        for id in invalid_ids {
            assert!(!is_valid_device_id(id), "Device ID {} should be invalid", id);
        }
    }
}

// Helper functions for validation
fn validate_security_score(score: i32) -> bool {
    score >= 0 && score <= 100
}

fn is_valid_device_id(id: &str) -> bool {
    // Device ID should be uppercase, contain hyphens, and have at least one number
    !id.is_empty() 
        && id.chars().all(|c| c.is_uppercase() || c.is_numeric() || c == '-')
        && id.contains('-')
        && id.chars().any(|c| c.is_numeric())
}
