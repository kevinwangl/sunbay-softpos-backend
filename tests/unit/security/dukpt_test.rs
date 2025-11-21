// Unit tests for DUKPT encryption/decryption
use crate::security::dukpt::{DukptManager, derive_key};
use hex;

#[cfg(test)]
mod dukpt_tests {
    use super::*;

    #[test]
    fn test_dukpt_key_derivation() {
        // Test key derivation with known values
        let bdk = hex::decode("0123456789ABCDEFFEDCBA9876543210").unwrap();
        let ksn = hex::decode("FFFF9876543210E00000").unwrap();

        let derived_key = derive_key(&bdk, &ksn);
        assert!(derived_key.is_ok());
        
        let key = derived_key.unwrap();
        assert_eq!(key.len(), 16); // 128-bit key
    }

    #[test]
    fn test_dukpt_encryption_decryption() {
        let manager = DukptManager::new();
        let plaintext = b"1234567890123456"; // 16-byte card number
        let ksn = hex::decode("FFFF9876543210E00000").unwrap();

        // Encrypt
        let encrypted = manager.encrypt(plaintext, &ksn);
        assert!(encrypted.is_ok());
        
        let ciphertext = encrypted.unwrap();
        assert_ne!(ciphertext, plaintext);

        // Decrypt
        let decrypted = manager.decrypt(&ciphertext, &ksn);
        assert!(decrypted.is_ok());
        assert_eq!(decrypted.unwrap(), plaintext);
    }

    #[test]
    fn test_ksn_counter_increment() {
        let mut ksn = hex::decode("FFFF9876543210E00000").unwrap();
        let original_ksn = ksn.clone();

        // Increment KSN counter
        increment_ksn_counter(&mut ksn);
        
        // Verify counter was incremented
        assert_ne!(ksn, original_ksn);
        assert_eq!(ksn.len(), original_ksn.len());
    }

    #[test]
    fn test_invalid_ksn_length() {
        let manager = DukptManager::new();
        let plaintext = b"1234567890123456";
        let invalid_ksn = hex::decode("FFFF98765432").unwrap(); // Too short

        let result = manager.encrypt(plaintext, &invalid_ksn);
        assert!(result.is_err());
    }

    #[test]
    fn test_pin_block_format() {
        // Test PIN block formatting (ISO Format 0)
        let pin = "1234";
        let pan = "1234567890123456";

        let pin_block = format_pin_block(pin, pan);
        assert!(pin_block.is_ok());
        
        let block = pin_block.unwrap();
        assert_eq!(block.len(), 8); // 64-bit PIN block
    }

    #[test]
    fn test_multiple_transactions_different_keys() {
        let manager = DukptManager::new();
        let plaintext = b"1234567890123456";
        
        let ksn1 = hex::decode("FFFF9876543210E00000").unwrap();
        let ksn2 = hex::decode("FFFF9876543210E00001").unwrap();

        let encrypted1 = manager.encrypt(plaintext, &ksn1).unwrap();
        let encrypted2 = manager.encrypt(plaintext, &ksn2).unwrap();

        // Different KSNs should produce different ciphertexts
        assert_ne!(encrypted1, encrypted2);
    }
}

// Helper functions
fn increment_ksn_counter(ksn: &mut [u8]) {
    // Increment the counter portion of the KSN (last 3 bytes)
    let counter_start = ksn.len() - 3;
    let mut carry = 1u16;
    
    for i in (counter_start..ksn.len()).rev() {
        let sum = ksn[i] as u16 + carry;
        ksn[i] = (sum & 0xFF) as u8;
        carry = sum >> 8;
        if carry == 0 {
            break;
        }
    }
}

fn format_pin_block(pin: &str, pan: &str) -> Result<Vec<u8>, String> {
    if pin.len() < 4 || pin.len() > 12 {
        return Err("Invalid PIN length".to_string());
    }
    
    if pan.len() != 16 {
        return Err("Invalid PAN length".to_string());
    }

    // ISO Format 0: 0 | PIN length | PIN | padding | XOR with PAN
    let mut block = vec![0u8; 8];
    block[0] = 0x00 | (pin.len() as u8);
    
    // Add PIN digits
    for (i, digit) in pin.chars().enumerate() {
        let byte_idx = 1 + i / 2;
        let nibble = digit.to_digit(10).ok_or("Invalid PIN digit")? as u8;
        if i % 2 == 0 {
            block[byte_idx] |= nibble << 4;
        } else {
            block[byte_idx] |= nibble;
        }
    }
    
    // Pad with 0xF
    for i in (1 + pin.len() / 2)..8 {
        block[i] = 0xFF;
    }

    Ok(block)
}
