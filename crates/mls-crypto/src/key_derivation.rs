//! Key derivation functions

use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroizing;

/// Argon2 parameters for production use
/// m=64MiB, t=3, p=4
/// These parameters provide a good balance between security and performance
/// on modern hardware (2024+).
///
/// Memory cost of 64 MiB ensures resistance against GPU-based brute force attacks.
/// 3 iterations and 4 parallel threads provide good performance on modern CPUs.
const PROD_ARGON2_MEMORY_COST: u32 = 65536; // 64 MiB
const PROD_ARGON2_ITERATIONS: u32 = 3;
const PROD_ARGON2_PARALLELISM: u32 = 4;

/// Argon2 parameters for testing - reduced for faster test execution
/// m=1MiB, t=1, p=1
const TEST_ARGON2_MEMORY_COST: u32 = 1024; // 1 MiB
const TEST_ARGON2_ITERATIONS: u32 = 1;
const TEST_ARGON2_PARALLELISM: u32 = 1;

/// Derive master key from password using Argon2id
///
/// Uses production parameters (m=64MiB, t=3, p=4) for security.
/// When compiled with the `fast-tests` feature, uses reduced parameters (m=1MiB, t=1, p=1)
/// for faster test execution.
///
/// # Errors
/// Returns an error if the Argon2 parameters are invalid or the hashing fails.
pub fn derive_master_key(
    password: &str,
    salt: &[u8],
) -> Result<Zeroizing<[u8; 32]>, crate::error::CryptoError> {
    use argon2::{Algorithm, Argon2, Params, Version};

    let memory_cost = if cfg!(feature = "fast-tests") {
        TEST_ARGON2_MEMORY_COST
    } else {
        PROD_ARGON2_MEMORY_COST
    };
    let iterations = if cfg!(feature = "fast-tests") {
        TEST_ARGON2_ITERATIONS
    } else {
        PROD_ARGON2_ITERATIONS
    };
    let parallelism = if cfg!(feature = "fast-tests") {
        TEST_ARGON2_PARALLELISM
    } else {
        PROD_ARGON2_PARALLELISM
    };

    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::default(),
        Params::new(
            memory_cost,
            iterations,
            parallelism,
            Some(32),
        )?,
    );

    let mut key = Zeroizing::new([0u8; 32]);
    argon2.hash_password_into(password.as_bytes(), salt, &mut *key)?;
    Ok(key)
}

/// Derive a key from a master key using HKDF
///
/// # Errors
/// Returns an error if the HKDF expansion fails.
pub fn derive_key(
    master_key: &Zeroizing<[u8; 32]>,
    context: &[u8],
) -> Result<Zeroizing<[u8; 32]>, crate::error::CryptoError> {
    let master_key_bytes: &[u8; 32] = master_key;
    let hkdf = Hkdf::<Sha256>::new(None, master_key_bytes);
    let mut derived_key = Zeroizing::new([0u8; 32]);
    hkdf.expand(context, &mut *derived_key)
        .map_err(|_| crate::error::CryptoError::HkdfError)?;
    Ok(derived_key)
}
