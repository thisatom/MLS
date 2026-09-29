//! Property-based tests for mls-crypto using proptest

use mls_crypto::traits::{AesGcmBackend, CryptoBackend};
use proptest::prelude::*;
use proptest::test_runner::Config;
use secrecy::ExposeSecret;

// Configure proptest to use fewer test cases for faster execution
fn proptest_config() -> Config {
    Config::with_cases(3)
}

// Generate a valid password for testing (non-empty, alphanumeric)
prop_compose! {
    fn valid_password()(
        _size in 1..20usize,
    )(password in "[a-zA-Z0-9]{1,20}") -> String {
        password
    }
}

// Generate a valid salt for testing (16 bytes fixed for speed)
prop_compose! {
    fn valid_salt()(
        salt in any::<[u8; 16]>()
    ) -> Vec<u8> {
        salt.to_vec()
    }
}

// Generate arbitrary plaintext data (0-100 bytes for speed)
prop_compose! {
    fn valid_plaintext()(
        data in any::<Vec<u8>>().prop_filter(
            "Plaintext must be 0-100 bytes",
            |d| d.len() <= 100,
        ),
    ) -> Vec<u8> {
        data
    }
}

// Test that master key derivation is deterministic
proptest! {
    #[test]
    fn prop_master_key_deterministic(password in valid_password(), salt in valid_salt()) {
        let backend = AesGcmBackend;
        
        let key1 = backend.derive_master_key(&password, &salt).unwrap();
        let key2 = backend.derive_master_key(&password, &salt).unwrap();
        
        assert_eq!(&*key1, &*key2);
    }
}

// Test that different passwords produce different master keys
proptest! {
    #![proptest_config(proptest_config())]
    #[test]
    fn prop_different_passwords_different_keys(
        password1 in valid_password(),
        password2 in valid_password(),
        salt in valid_salt(),
    ) {
        // Skip if passwords are the same
        prop_assume!(password1 != password2);
        
        let backend = AesGcmBackend;
        
        let key1 = backend.derive_master_key(&password1, &salt).unwrap();
        let key2 = backend.derive_master_key(&password2, &salt).unwrap();
        
        assert_ne!(&*key1, &*key2);
    }
}

// Test that different salts produce different master keys
proptest! {
    #![proptest_config(proptest_config())]
    #[test]
    fn prop_different_salts_different_keys(
        password in valid_password(),
        salt1 in valid_salt(),
        salt2 in valid_salt(),
    ) {
        // Skip if salts are the same
        prop_assume!(salt1 != salt2);
        
        let backend = AesGcmBackend;
        
        let key1 = backend.derive_master_key(&password, &salt1).unwrap();
        let key2 = backend.derive_master_key(&password, &salt2).unwrap();
        
        assert_ne!(&*key1, &*key2);
    }
}

// Test round-trip encryption/decryption for arbitrary data
proptest! {
    #![proptest_config(proptest_config())]
    #[test]
    fn prop_encrypt_decrypt_roundtrip(
        password in valid_password(),
        salt in valid_salt(),
        plaintext in valid_plaintext(),
    ) {
        let backend = AesGcmBackend;
        
        let master_key = backend.derive_master_key(&password, &salt).unwrap();
        let enc_key = backend.derive_encryption_key(&master_key).unwrap();
        
        let (nonce, ciphertext) = backend.encrypt(&enc_key, &plaintext).unwrap();
        let decrypted = backend.decrypt(&enc_key, &nonce, &ciphertext).unwrap();
        
        assert_eq!(decrypted.expose_secret(), &plaintext);
    }
}

// Test that derived keys are different
proptest! {
    #![proptest_config(proptest_config())]
    #[test]
    fn prop_derived_keys_are_different(
        password in valid_password(),
        salt in valid_salt(),
    ) {
        let backend = AesGcmBackend;
        
        let master_key = backend.derive_master_key(&password, &salt).unwrap();
        let auth_key = backend.derive_auth_key(&master_key).unwrap();
        let enc_key = backend.derive_encryption_key(&master_key).unwrap();
        
        // Auth key and encryption key should be different
        assert_ne!(&*auth_key, &*enc_key);
        // Both should be different from master key
        assert_ne!(&*auth_key, &*master_key);
        assert_ne!(&*enc_key, &*master_key);
    }
}
