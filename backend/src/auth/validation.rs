use rustrict::CensorStr;

/// Reserved handles that cannot be registered (checked case-insensitively).
const RESERVED_HANDLES: &[&str] = &["sysop", "admin", "guest", "system", "operator"];

/// Validate a user handle (display name).
///
/// Rules:
/// - 3-20 characters after trimming
/// - Only alphanumeric characters and spaces
/// - Cannot start or end with a space
/// - No consecutive spaces
/// - Not a reserved name
/// - No profanity (including leetspeak variants via rustrict)
pub fn validate_handle(handle: &str) -> Result<(), String> {
    let trimmed = handle.trim();

    // Length check
    if trimmed.len() < 3 {
        return Err("Handle must be at least 3 characters".to_string());
    }
    if trimmed.len() > 20 {
        return Err("Handle must be 20 characters or less".to_string());
    }

    // Character set: alphanumeric + spaces only
    if !trimmed.chars().all(|c| c.is_alphanumeric() || c == ' ') {
        return Err("Handle may only contain letters, numbers, and spaces".to_string());
    }

    // Cannot start or end with space (after trimming this checks the original)
    if handle.starts_with(' ') || handle.ends_with(' ') {
        return Err("Handle cannot start or end with a space".to_string());
    }

    // No consecutive spaces
    if trimmed.contains("  ") {
        return Err("Handle cannot contain consecutive spaces".to_string());
    }

    // Reserved names
    let lower = trimmed.to_lowercase();
    if RESERVED_HANDLES.contains(&lower.as_str()) {
        return Err("That handle is reserved".to_string());
    }

    // Profanity filter (rustrict catches leetspeak variants)
    if trimmed.is_inappropriate() {
        return Err("Handle contains inappropriate language".to_string());
    }

    Ok(())
}

/// Validate an email address (basic structural validation).
///
/// Rules:
/// - Contains exactly one '@'
/// - Has characters before and after '@'
/// - Domain part contains at least one '.'
/// - Total length <= 254 characters
pub fn validate_email(email: &str) -> Result<(), String> {
    if email.len() > 254 {
        return Err("Email address is too long".to_string());
    }

    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err("Email must contain exactly one '@'".to_string());
    }

    let local = parts[0];
    let domain = parts[1];

    if local.is_empty() {
        return Err("Email must have characters before '@'".to_string());
    }
    if domain.is_empty() {
        return Err("Email must have characters after '@'".to_string());
    }
    if !domain.contains('.') {
        return Err("Email domain must contain a '.'".to_string());
    }

    Ok(())
}

/// Validate a password.
///
/// Rules:
/// - Minimum 6 characters
/// - Maximum 128 characters
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 6 {
        return Err("Password must be at least 6 characters".to_string());
    }
    if password.len() > 128 {
        return Err("Password must be 128 characters or less".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Handle validation tests ---

    #[test]
    fn valid_handle_passes() {
        assert!(validate_handle("CoolUser").is_ok());
        assert!(validate_handle("The Doctor").is_ok());
        assert!(validate_handle("abc").is_ok());
        assert!(validate_handle("Abcdefghij0123456789").is_ok()); // 20 chars
    }

    #[test]
    fn handle_too_short_fails() {
        assert!(validate_handle("ab").is_err());
        assert!(validate_handle("").is_err());
    }

    #[test]
    fn handle_too_long_fails() {
        assert!(validate_handle("a]234567890123456789012345").is_err());
    }

    #[test]
    fn handle_invalid_characters_fails() {
        assert!(validate_handle("user@name").is_err());
        assert!(validate_handle("user!").is_err());
        assert!(validate_handle("user_name").is_err());
    }

    #[test]
    fn handle_space_rules() {
        // Leading/trailing spaces fail
        assert!(validate_handle(" User").is_err());
        assert!(validate_handle("User ").is_err());

        // Consecutive spaces fail
        assert!(validate_handle("Two  Spaces").is_err());

        // Single internal space is ok
        assert!(validate_handle("Two Spaces").is_ok());
    }

    #[test]
    fn handle_reserved_names_fail() {
        assert!(validate_handle("SysOp").is_err());
        assert!(validate_handle("admin").is_err());
        assert!(validate_handle("GUEST").is_err());
        assert!(validate_handle("system").is_err());
        assert!(validate_handle("Operator").is_err());
    }

    #[test]
    fn handle_profanity_fails() {
        // rustrict should catch common profanity
        assert!(validate_handle("shithead").is_err());
    }

    // --- Email validation tests ---

    #[test]
    fn valid_email_passes() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("a@b.c").is_ok());
    }

    #[test]
    fn email_missing_at_fails() {
        assert!(validate_email("userexample.com").is_err());
    }

    #[test]
    fn email_multiple_at_fails() {
        assert!(validate_email("user@@example.com").is_err());
    }

    #[test]
    fn email_missing_local_fails() {
        assert!(validate_email("@example.com").is_err());
    }

    #[test]
    fn email_missing_domain_fails() {
        assert!(validate_email("user@").is_err());
    }

    #[test]
    fn email_domain_no_dot_fails() {
        assert!(validate_email("user@localhost").is_err());
    }

    #[test]
    fn email_too_long_fails() {
        let long_email = format!("{}@example.com", "a".repeat(250));
        assert!(validate_email(&long_email).is_err());
    }

    // --- Password validation tests ---

    #[test]
    fn valid_password_passes() {
        assert!(validate_password("abcdef").is_ok());
        assert!(validate_password("correct-horse-battery-staple").is_ok());
    }

    #[test]
    fn password_too_short_fails() {
        assert!(validate_password("12345").is_err());
        assert!(validate_password("").is_err());
    }

    #[test]
    fn password_too_long_fails() {
        let long_pw = "a".repeat(129);
        assert!(validate_password(&long_pw).is_err());
    }
}
