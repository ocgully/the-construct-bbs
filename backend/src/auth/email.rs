use crate::config::EmailConfig;
use rand::Rng;

/// Generate a random 6-digit verification code, zero-padded.
pub fn generate_verification_code() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1000000))
}

/// Send a verification email via SMTP, or log the code if SMTP is not configured.
///
/// When `config` is `None` (development mode), the verification code is printed
/// to stdout so the developer can retrieve it from server logs.
///
/// When `config` is `Some`, lettre is used to send the email via SMTP. The
/// actual send is spawned onto a background task so it does not block the
/// registration flow. Errors are logged but do not fail the operation.
pub async fn send_verification_email(
    config: &Option<EmailConfig>,
    to_email: &str,
    code: &str,
    bbs_name: &str,
) -> Result<(), String> {
    match config {
        None => {
            // Development mode -- print code to console
            println!(
                "[EMAIL] SMTP not configured. Verification code for {}: {}",
                to_email, code
            );
            Ok(())
        }
        Some(cfg) => {
            use lettre::message::Mailbox;
            use lettre::transport::smtp::authentication::Credentials;
            use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

            let from_mailbox: Mailbox = format!("{} <{}>", cfg.from_name, cfg.from_address)
                .parse()
                .map_err(|e| format!("Invalid from address: {}", e))?;

            let to_mailbox: Mailbox = to_email
                .parse()
                .map_err(|e| format!("Invalid to address: {}", e))?;

            let body = format!(
                "Your verification code is: {}\n\n\
                 This code expires in 24 hours.\n\n\
                 If you did not request this, please ignore this email.",
                code
            );

            let email = Message::builder()
                .from(from_mailbox)
                .to(to_mailbox)
                .subject(format!("{} - Email Verification Code", bbs_name))
                .body(body)
                .map_err(|e| format!("Failed to build email: {}", e))?;

            let creds = Credentials::new(
                cfg.smtp_username.clone(),
                cfg.smtp_password.clone(),
            );

            let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.smtp_host)
                .map_err(|e| format!("SMTP relay error: {}", e))?
                .port(cfg.smtp_port)
                .credentials(creds)
                .build();

            // Spawn email sending so it doesn't block registration
            let to_email_owned = to_email.to_string();
            tokio::spawn(async move {
                match mailer.send(email).await {
                    Ok(_) => {
                        println!("[EMAIL] Verification email sent to {}", to_email_owned);
                    }
                    Err(e) => {
                        eprintln!(
                            "[EMAIL] Failed to send verification email to {}: {}",
                            to_email_owned, e
                        );
                    }
                }
            });

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verification_code_is_six_digits() {
        for _ in 0..100 {
            let code = generate_verification_code();
            assert_eq!(code.len(), 6, "code should be 6 characters: {}", code);
            assert!(
                code.chars().all(|c| c.is_ascii_digit()),
                "code should be all digits: {}",
                code
            );
        }
    }

    #[test]
    fn verification_code_is_zero_padded() {
        // Generate many codes and check format -- at least some should start with 0
        // statistically (about 10% chance per code). 100 samples should hit it.
        let codes: Vec<String> = (0..100).map(|_| generate_verification_code()).collect();
        // All should be exactly 6 digits
        for code in &codes {
            assert_eq!(code.len(), 6);
            assert!(code.parse::<u32>().is_ok());
        }
    }

    #[tokio::test]
    async fn send_email_without_config_succeeds() {
        // Development mode: no SMTP config, should print to console and return Ok
        let result =
            send_verification_email(&None, "test@example.com", "123456", "Test BBS").await;
        assert!(result.is_ok(), "should succeed without SMTP config");
    }
}
