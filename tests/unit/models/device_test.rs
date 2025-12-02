// Unit tests for Device model
use crate::models::device::{Device, DeviceMode, DeviceStatus, TeeType};
use chrono::Utc;
use uuid::Uuid;

#[cfg(test)]
mod device_tests {
    use super::*;

    #[test]
    fn test_device_creation() {
        let device = Device {
            id: Uuid::new_v4().to_string(),
            imei: "123456789012345".to_string(),
            model: "V2PRO".to_string(),
            os_version: "12.0".to_string(),
            tee_type: TeeType::TrustZone.as_str().to_string(),
            device_mode: DeviceMode::FullPos.as_str().to_string(),
            public_key: vec![1, 2, 3],
            status: DeviceStatus::Pending.as_str().to_string(),
            merchant_id: None,
            merchant_name: None,
            security_score: 85,
            current_ksn: "".to_string(),
            ipek_injected_at: None,
            key_remaining_count: 0,
            key_total_count: 0,
            registered_at: Utc::now().to_rfc3339(),
            approved_at: None,
            approved_by: None,
            last_active_at: None,
            updated_at: Utc::now().to_rfc3339(),
            nfc_present: true,
        };

        assert_eq!(device.imei, "123456789012345");
        assert_eq!(device.status, DeviceStatus::Pending.as_str());
        assert_eq!(device.security_score, 85);
    }

    #[test]
    fn test_device_status_transitions() {
        let mut device = Device {
            id: Uuid::new_v4().to_string(),
            imei: "123456789012345".to_string(),
            model: "V2PRO".to_string(),
            os_version: "1.0.0".to_string(),
            tee_type: TeeType::TrustZone.as_str().to_string(),
            device_mode: DeviceMode::FullPos.as_str().to_string(),
            public_key: vec![1, 2, 3],
            status: DeviceStatus::Pending.as_str().to_string(),
            merchant_id: None,
            merchant_name: None,
            security_score: 100,
            current_ksn: "".to_string(),
            ipek_injected_at: None,
            key_remaining_count: 0,
            key_total_count: 0,
            registered_at: Utc::now().to_rfc3339(),
            approved_at: None,
            approved_by: None,
            last_active_at: None,
            updated_at: Utc::now().to_rfc3339(),
            nfc_present: true,
        };

        assert_eq!(device.status, DeviceStatus::Pending.as_str());

        // Change status
        device.status = DeviceStatus::Active.as_str().to_string();
        assert_eq!(device.status, DeviceStatus::Active.as_str());

        // Change status again
        device.status = DeviceStatus::Suspended.as_str().to_string();
        assert_eq!(device.status, DeviceStatus::Suspended.as_str());
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
        let valid_ids = vec!["DEVICE-001", "TEST-DEVICE-123", "PROD-DEVICE-999"];

        for id in valid_ids {
            assert!(is_valid_device_id(id), "Device ID {} should be valid", id);
        }

        let invalid_ids = vec![
            "",
            "DEVICE",
            "device-001", // lowercase
            "DEVICE_001", // underscore instead of hyphen
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
