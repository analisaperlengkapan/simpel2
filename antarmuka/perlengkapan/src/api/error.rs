use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AppError {
    Network(String),
    Auth(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    Validation(Vec<FieldError>),
    Server(String),
    Parse(String),
    Unknown(String),
}

impl AppError {
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth(msg.into())
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    pub fn server(msg: impl Into<String>) -> Self {
        Self::Server(msg.into())
    }

    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }

    pub fn unknown(msg: impl Into<String>) -> Self {
        Self::Unknown(msg.into())
    }

    pub fn validation(errors: Vec<FieldError>) -> Self {
        Self::Validation(errors)
    }

    pub fn from_status(status: u16, body: &str) -> Self {
        match status {
            401 => Self::Auth("Sesi Anda telah berakhir. Silakan login kembali.".to_string()),
            403 => Self::Forbidden(
                "Anda tidak memiliki hak akses untuk melakukan tindakan ini.".to_string(),
            ),
            404 => Self::NotFound("Data yang diminta tidak ditemukan.".to_string()),
            409 => Self::Conflict(if body.is_empty() {
                "Terjadi konflik data.".to_string()
            } else {
                body.to_string()
            }),
            422 => Self::Validation(parse_validation_body(body)),
            400 => Self::Validation(parse_validation_body(body)),
            500..=599 => Self::Server(format!("Kesalahan server ({status}): {body}")),
            _ => Self::Unknown(format!("HTTP {status}: {body}")),
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::Network(_) => {
                "Gagal terhubung ke server. Periksa koneksi internet Anda.".to_string()
            }
            Self::Auth(msg) => msg.clone(),
            Self::Forbidden(msg) => msg.clone(),
            Self::NotFound(msg) => msg.clone(),
            Self::Conflict(msg) => msg.clone(),
            Self::Validation(errors) => {
                if errors.is_empty() {
                    "Data yang dikirim tidak valid.".to_string()
                } else {
                    errors
                        .iter()
                        .map(|e| format!("{}: {}", e.field, e.message))
                        .collect::<Vec<_>>()
                        .join("; ")
                }
            }
            Self::Server(_) => {
                "Terjadi kesalahan di server. Silakan coba lagi beberapa saat.".to_string()
            }
            Self::Parse(_) => "Format respons server tidak dapat diproses.".to_string(),
            Self::Unknown(msg) => msg.clone(),
        }
    }

    pub fn is_auth_error(&self) -> bool {
        matches!(self, Self::Auth(_))
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(s) => write!(f, "Network error: {s}"),
            Self::Auth(s) => write!(f, "Auth error: {s}"),
            Self::Forbidden(s) => write!(f, "Forbidden: {s}"),
            Self::NotFound(s) => write!(f, "Not found: {s}"),
            Self::Conflict(s) => write!(f, "Conflict: {s}"),
            Self::Validation(errors) => {
                write!(f, "Validation error: ")?;
                for (i, e) in errors.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}: {}", e.field, e.message)?;
                }
                Ok(())
            }
            Self::Server(s) => write!(f, "Server error: {s}"),
            Self::Parse(s) => write!(f, "Parse error: {s}"),
            Self::Unknown(s) => write!(f, "Unknown error: {s}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        Self::Unknown(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        Self::Unknown(s.to_string())
    }
}

impl From<gloo_net::Error> for AppError {
    fn from(err: gloo_net::Error) -> Self {
        match err {
            gloo_net::Error::JsError(e) => Self::Network(e.message),
            gloo_net::Error::SerdeError(e) => Self::Parse(e.to_string()),
            gloo_net::Error::GlooError(msg) => Self::Network(msg),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::Parse(err.to_string())
    }
}

impl From<url::ParseError> for AppError {
    fn from(err: url::ParseError) -> Self {
        Self::Unknown(format!("URL parse error: {err}"))
    }
}

fn parse_validation_body(body: &str) -> Vec<FieldError> {
    if body.is_empty() {
        return Vec::new();
    }
    if let Ok(errors) = serde_json::from_str::<Vec<FieldError>>(body) {
        return errors;
    }
    if let Ok(wrapper) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(errors) = wrapper.get("errors").and_then(|v| v.as_array()) {
            return errors
                .iter()
                .filter_map(|e| {
                    let field = e.get("field")?.as_str()?.to_string();
                    let message = e.get("message")?.as_str()?.to_string();
                    Some(FieldError { field, message })
                })
                .collect();
        }
        if let Some(msg) = wrapper.get("message").and_then(|v| v.as_str()) {
            return vec![FieldError {
                field: "general".to_string(),
                message: msg.to_string(),
            }];
        }
    }
    vec![FieldError {
        field: "general".to_string(),
        message: body.to_string(),
    }]
}

pub type AppResult<T> = Result<T, AppError>;
