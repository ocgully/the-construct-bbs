use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};

/// Create an Argon2id hasher with OWASP 2026 recommended parameters:
/// m=19456 KiB (19 MiB), t=2 iterations, p=1 parallelism
fn hasher() -> Argon2<'static> {
    let params = Params::new(19456, 2, 1, None).expect("valid Argon2 params");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

/// Hash a password using Argon2id with OWASP 2026 parameters.
///
/// Returns a PHC-formatted hash string suitable for storage.
/// This is a CPU-intensive blocking function -- callers should use
/// `tokio::task::spawn_blocking()` when calling from async context.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = hasher().hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verify a password against a stored PHC-formatted hash.
///
/// Returns `Ok(true)` if the password matches, `Ok(false)` if it does not.
/// Only returns `Err` for structural issues (e.g., malformed hash string).
/// This is a CPU-intensive blocking function -- callers should use
/// `tokio::task::spawn_blocking()` when calling from async context.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed = PasswordHash::new(hash)?;
    match hasher().verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_succeeds() {
        let password = "correct-horse-battery-staple";
        let hash = hash_password(password).expect("hashing should succeed");

        // Hash should be PHC-formatted and contain argon2id
        assert!(hash.starts_with("$argon2id$"));

        let result = verify_password(password, &hash).expect("verify should not error");
        assert!(result, "correct password should verify as true");
    }

    #[test]
    fn verify_wrong_password_returns_false() {
        let hash = hash_password("right-password").expect("hashing should succeed");
        let result = verify_password("wrong-password", &hash).expect("verify should not error");
        assert!(!result, "wrong password should verify as false");
    }

    #[test]
    fn hash_produces_different_salts() {
        let h1 = hash_password("same").expect("hash 1");
        let h2 = hash_password("same").expect("hash 2");
        assert_ne!(h1, h2, "two hashes of same password should differ (different salts)");
    }

    #[test]
    fn verify_malformed_hash_returns_error() {
        let result = verify_password("pw", "not-a-valid-hash");
        assert!(result.is_err(), "malformed hash should return Err");
    }
}
