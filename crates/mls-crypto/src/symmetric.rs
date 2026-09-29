//! Symmetric encryption functions

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm,
};
use generic_array::typenum::U12;
use generic_array::GenericArray;
use secrecy::SecretVec;
use zeroize::Zeroizing;

/// Encrypt data using AES-256-GCM
///
/// # Errors
/// Returns an error if the encryption fails.
pub fn encrypt(
    key: &Zeroizing<[u8; 32]>,
    plaintext: &[u8],
) -> Result<(GenericArray<u8, U12>, Vec<u8>), crate::error::CryptoError> {
    let key_array: &[u8; 32] = key;
    let cipher = Aes256Gcm::new(GenericArray::from_slice(key_array));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext)?;
    Ok((nonce, ciphertext))
}

/// Decrypt data using AES-256-GCM
///
/// # Errors
/// Returns an error if the decryption fails.
pub fn decrypt(
    key: &Zeroizing<[u8; 32]>,
    nonce: &GenericArray<u8, U12>,
    ciphertext: &[u8],
) -> Result<SecretVec<u8>, crate::error::CryptoError> {
    let key_array: &[u8; 32] = key;
    let cipher = Aes256Gcm::new(GenericArray::from_slice(key_array));
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(SecretVec::new(plaintext))
}
