// Unit tests for DUKPT
use crate::security::DukptKeyDerivation;

#[cfg(test)]
mod dukpt_tests {
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
}
