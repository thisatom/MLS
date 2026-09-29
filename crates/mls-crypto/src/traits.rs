//! Crypto backend traits for MLS
//!
//! This module provides the `CryptoBackend` trait that allows swapping
//! cryptographic implementations (e.g., for post-quantum cryptography support).

use secrecy::SecretVec;
use zeroize::Zeroizing;

/// Trait for cryptographic backend implementations
///
/// This trait allows swapping cryptographic implementations (e.g., for PQC support).
/// The default implementation uses AES-256-GCM for symmetric encryption and
/// Argon2id/HKDF for key derivation.
pub trait CryptoBackend {
    /// Derive a master key from a password and salt using Argon2id
    ///
    /// # Arguments
    /// * `password` - User's master password
    /// * `salt` - Random salt for key derivation
    ///
    /// # Returns
    /// 32-byte master key
    ///
    /// # Errors
    /// Returns an error if Argon2 parameters are invalid or hashing fails.
    fn derive_master_key(
        &self,
        password: &str,
        salt: &[u8],
    ) -> Result<Zeroizing<[u8; 32]>, crate::error::CryptoError>;

    /// Derive an authentication key from the master key using HKDF
    ///
    /// # Arguments
    /// * `master_key` - 32-byte master key
    ///
    /// # Returns
    /// 32-byte authentication key
    ///
    /// # Errors
    /// Returns an error if HKDF expansion fails.
    fn derive_auth_key(
        &self,
        master_key: &Zeroizing<[u8; 32]>,
    ) -> Result<Zeroizing<[u8; 32]>, crate::error::CryptoError>;

    /// Derive an encryption key from the master key using HKDF
    ///
    /// # Arguments
    /// * `master_key` - 32-byte master key
    ///
    /// # Returns
    /// 32-byte encryption key
    ///
    /// # Errors
    /// Returns an error if HKDF expansion fails.
    fn derive_encryption_key(
        &self,
        master_key: &Zeroizing<[u8; 32]>,
    ) -> Result<Zeroizing<[u8; 32]>, crate::error::CryptoError>;

    /// Encrypt data using symmetric encryption
    ///
    /// # Arguments
    /// * `key` - 32-byte encryption key
    /// * `plaintext` - Data to encrypt
    ///
    /// # Returns
    /// Tuple of (`nonce_bytes`, ciphertext) where nonce is a byte vector
    ///
    /// # Errors
    /// Returns an error if encryption fails.
    fn encrypt(
        &self,
        key: &Zeroizing<[u8; 32]>,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>), crate::error::CryptoError>;

    /// Decrypt data using symmetric encryption
    ///
    /// # Arguments
    /// * `key` - 32-byte encryption key
    /// * `nonce` - Nonce bytes used for encryption
    /// * `ciphertext` - Encrypted data
    ///
    /// # Returns
    /// Decrypted plaintext
    ///
    /// # Errors
    /// Returns an error if decryption fails (authentication error).
    fn decrypt(
        &self,
        key: &Zeroizing<[u8; 32]>,
        nonce: &[u8],
        ciphertext: &[u8],
    ) -> Result<SecretVec<u8>, crate::error::CryptoError>;
}

/// Default AES-GCM backend implementation
#[derive(Debug, Default, Clone, Copy)]
pub struct AesGcmBackend;

impl CryptoBackend for AesGcmBackend {
    fn derive_master_key(
        &self,
        password: &str,
        salt: &[u8],
    ) -> Result<Zeroizing<[u8; 32]>, crate::error::CryptoError> {
        super::key_derivation::derive_master_key(password, salt)
    }

    fn derive_auth_key(
        &self,
        master_key: &Zeroizing<[u8; 32]>,
    ) -> Result<Zeroizing<[u8; 32]>, crate::error::CryptoError> {
        super::key_derivation::derive_key(master_key, b"auth")
    }

    fn derive_encryption_key(
        &self,
        master_key: &Zeroizing<[u8; 32]>,
    ) -> Result<Zeroizing<[u8; 32]>, crate::error::CryptoError> {
        super::key_derivation::derive_key(master_key, b"encryption")
    }

    fn encrypt(
        &self,
        key: &Zeroizing<[u8; 32]>,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>), crate::error::CryptoError> {
        let (nonce, ciphertext) = super::symmetric::encrypt(key, plaintext)?;
        Ok ((nonce.to_vec(), ciphertext))
    }

    fn decrypt(
        &self,
        key: &Zeroizing<[u8; 32]>,
        nonce: &[u8],
        ciphertext: &[u8],
    ) -> Result<SecretVec<u8>, crate::error::CryptoError> {
        use aes_gcm::Nonce;
        
        let nonce_typed = Nonce::from_slice(nonce);
        super::symmetric::decrypt(key, nonce_typed, ciphertext)
    }
}

#[cfg(test)]
mod tests {
    use secrecy::ExposeSecret;
    use super::*;

    // --- Property-based tests ---
    
    /// Test that encrypt/decrypt is idempotent for any data
    #[test]
    fn test_encrypt_decrypt_idempotent() {
        let backend = AesGcmBackend;
        let password = "test_password_123";
        let salt = b"salt_123456789012";

        let master_key = backend.derive_master_key(password, salt).unwrap();
        let enc_key = backend.derive_encryption_key(&master_key).unwrap();

        // Test with various data sizes
        let test_data = [
            vec![],
            vec![0u8],
            vec![1u8; 16],
            vec![2u8; 64],
            vec![3u8; 256],
            vec![4u8; 1024],
            (0..256).map(|i| (i % 256) as u8).collect(),
        ];

        for data in test_data {
            let (nonce, ciphertext) = backend.encrypt(&enc_key, &data).unwrap();
            let decrypted = backend.decrypt(&enc_key, &nonce, &ciphertext).unwrap();
            assert_eq!(decrypted.expose_secret(), &data);
        }
    }

    #[test]
    fn test_aes_gcm_backend_roundtrip() {
        let backend = AesGcmBackend;
        let password = "test_password";
        let salt = b"test_salt_12345678";

        // Derive keys
        let master_key = backend.derive_master_key(password, salt).unwrap();
        let enc_key = backend.derive_encryption_key(&master_key).unwrap();

        // Encrypt and decrypt
        let plaintext = b"Hello, World!";
        let (nonce, ciphertext) = backend.encrypt(&enc_key, plaintext).unwrap();
        let decrypted = backend.decrypt(&enc_key, &nonce, &ciphertext).unwrap();

        assert_eq!(decrypted.expose_secret(), plaintext);
    }

    #[test]
    fn test_different_passwords_different_keys() {
        let backend = AesGcmBackend;
        let salt = b"test_salt_12345678";

        let key1 = backend
            .derive_master_key("password1", salt)
            .unwrap();
        let key2 = backend
            .derive_master_key("password2", salt)
            .unwrap();

        // Keys should be different
        assert_ne!(&*key1, &*key2);
    }

    #[test]
    fn test_wrong_nonce_fails() {
        let backend = AesGcmBackend;
        let password = "test_password";
        let salt = b"test_salt_12345678";

        let master_key = backend.derive_master_key(password, salt).unwrap();
        let enc_key = backend.derive_encryption_key(&master_key).unwrap();

        let plaintext = b"Hello, World!";
        let (mut nonce, ciphertext) = backend.encrypt(&enc_key, plaintext).unwrap();

        // Create a wrong nonce by flipping one bit
        nonce[0] ^= 0x01;

        let result = backend.decrypt(&enc_key, &nonce, &ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_corrupted_ciphertext_fails() {
        let backend = AesGcmBackend;
        let password = "test_password";
        let salt = b"test_salt_12345678";

        let master_key = backend.derive_master_key(password, salt).unwrap();
        let enc_key = backend.derive_encryption_key(&master_key).unwrap();

        let plaintext = b"Hello, World!";
        let (nonce, mut ciphertext) = backend.encrypt(&enc_key, plaintext).unwrap();

        // Flip one bit in the ciphertext
        if !ciphertext.is_empty() {
            ciphertext[0] ^= 0x01;
        }

        let result = backend.decrypt(&enc_key, &nonce, &ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_key_fails() {
        let backend = AesGcmBackend;
        let salt = b"test_salt_12345678";

        let master_key1 = backend
            .derive_master_key("password1", salt)
            .unwrap();
        let enc_key1 = backend.derive_encryption_key(&master_key1).unwrap();

        let master_key2 = backend
            .derive_master_key("password2", salt)
            .unwrap();
        let enc_key2 = backend.derive_encryption_key(&master_key2).unwrap();

        let plaintext = b"Hello, World!";
        let (nonce, ciphertext) = backend.encrypt(&enc_key1, plaintext).unwrap();

        // Try to decrypt with wrong key
        let result = backend.decrypt(&enc_key2, &nonce, &ciphertext);
        assert!(result.is_err());
    }
}
