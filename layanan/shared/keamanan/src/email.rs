use lettre::{Message, SmtpTransport, Transport, transport::smtp::authentication::Credentials};
use std::env;

pub async fn send_email(to: &str, subject: &str, body: &str) -> Result<(), String> {
    let smtp_server = env::var("SMTP_SERVER").unwrap_or_default();
    let smtp_user = env::var("SMTP_USER").unwrap_or_default();
    let smtp_pass = env::var("SMTP_PASS").unwrap_or_default();
    let from = env::var("EMAIL_FROM").unwrap_or_else(|_| "noreply@simpelv2.local".to_string());
    if smtp_server.is_empty() {
        tracing::info!("[EMAIL] To: {} | Subject: {} | Body: {}", to, subject, body);
        return Ok(());
    }
    let email = Message::builder()
        .from(match from.parse() {
            Ok(addr) => addr,
            Err(_) => return Err(AppError::EmailError("Invalid from address".to_string())),
        })
        .to(match to.parse() {
            Ok(addr) => addr,
            Err(_) => return Err(AppError::EmailError("Invalid to address".to_string())),
        })
        .subject(subject)
        .body(body.to_string())
        .map_err(|e| e.to_string())?;
    let creds = Credentials::new(smtp_user, smtp_pass);
    let mailer = SmtpTransport::relay(&smtp_server)
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .build();
    mailer.send(&email).map_err(|e| e.to_string())?;
    Ok(())
} 