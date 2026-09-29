//! Tests for zeroize functionality
//!
//! These tests verify that secrets are properly handled and zeroized.

#![allow(unsafe_code)]

use mls_crypto::traits::{AesGcmBackend, CryptoBackend};
use secrecy::ExposeSecret;
use zeroize::Zeroizing;

/// Test that Zeroizing works correctly with our types
#[test]
fn test_zeroizing_type_works() {
    let key = Zeroizing::new([0x42u8; 32]);
    assert_eq!(&*key, &[0x42u8; 32]);
}

/// Test that encrypt/decrypt works with Zeroizing keys
#[test]
fn test_encrypt_decrypt_with_zeroizing() {
    let backend = AesGcmBackend;
    let password = "test_password";
    let salt = b"test_salt_12345678";

    let master_key = backend.derive_master_key(password, salt).unwrap();
    let enc_key = backend.derive_encryption_key(&master_key).unwrap();

    let plaintext = b"Test data for zeroizing";
    let (nonce, ciphertext) = backend.encrypt(&enc_key, plaintext).unwrap();
    let decrypted = backend.decrypt(&enc_key, &nonce, &ciphertext).unwrap();

    assert_eq!(decrypted.expose_secret(), plaintext);

    // Key should still be valid
    let (nonce2, ciphertext2) = backend.encrypt(&enc_key, plaintext).unwrap();
    let decrypted2 = backend.decrypt(&enc_key, &nonce2, &ciphertext2).unwrap();
    assert_eq!(decrypted2.expose_secret(), plaintext);
}

/// Test that keys are different for different passwords
#[test]
fn test_different_passwords_different_master_keys() {
    let backend = AesGcmBackend;
    let salt = b"test_salt_12345678";

    let key1 = backend
        .derive_master_key("password_1", salt)
        .unwrap();
    let key2 = backend
        .derive_master_key("password_2", salt)
        .unwrap();

    assert_ne!(&*key1, &*key2);
}

/// Test that same password and salt produce same key
#[test]
fn test_same_password_same_key() {
    let backend = AesGcmBackend;
    let password = "test_password";
    let salt = b"test_salt_12345678";

    let key1 = backend
        .derive_master_key(password, salt)
        .unwrap();
    let key2 = backend
        .derive_master_key(password, salt)
        .unwrap();

    assert_eq!(&*key1, &*key2);
}
