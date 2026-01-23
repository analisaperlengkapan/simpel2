//! Transform Secrets Engine - Production Implementation
//!
//! FF3-1 format-preserving encryption, tokenization, and masking for PCI/GDPR compliance.

use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use secreton_crypto::{FpeAlphabet, FpeEngine, FpeError, FpeKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// Transform engine errors
#[derive(Debug, thiserror::Error)]
pub enum TransformError {
    #[error("Transformation not found: {0}")]
    TransformationNotFound(String),
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    #[error("Invalid alphabet: {0}")]
    InvalidAlphabet(String),
    #[error("Encode failed: {0}")]
    EncodeFailed(String),
    #[error("Decode failed: {0}")]
    DecodeFailed(String),
    #[error("Invalid template: {0}")]
    InvalidTemplate(String),
    #[error("FPE error: {0}")]
    FpeError(#[from] FpeError),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Transformation already exists: {0}")]
    TransformationAlreadyExists(String),
    #[error("Role already exists: {0}")]
    RoleAlreadyExists(String),
    #[error("Access denied: role {0} cannot use transformation {1}")]
    AccessDenied(String, String),
}

/// Transformation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransformationType {
    FPE,
    Tokenization,
    Masking,
}

/// Masking pattern types
/// Requirement 10.3: Support credit card, email, phone masking patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MaskingPattern {
    /// Default masking (show last 4 characters)
    Default,
    /// Credit card masking (show last 4 digits)
    CreditCard,
    /// Email masking (show first char and domain)
    Email,
    /// Phone masking (show last 4 digits)
    Phone,
    /// Custom template-based masking
    Custom,
}

/// Alphabet for FPE
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Alphabet {
    Numeric,
    Alphanumeric,
    AlphanumericMixed,
    Custom(String),
}

impl Alphabet {
    pub fn to_fpe_alphabet(&self) -> FpeAlphabet {
        match self {
            Alphabet::Numeric => FpeAlphabet::Numeric,
            Alphabet::Alphanumeric => FpeAlphabet::Alphanumeric,
            Alphabet::AlphanumericMixed => FpeAlphabet::AlphanumericMixed,
            Alphabet::Custom(s) => FpeAlphabet::Custom(s.clone()),
        }
    }
}

/// Transformation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformation {
    pub name: String,
    pub transformation_type: TransformationType,
    pub template: Option<String>,
    pub alphabet: Option<Alphabet>,
    pub tweak_source: Option<String>,
    pub masking_char: Option<char>,
    pub masking_pattern: Option<MaskingPattern>,
    pub created_at: DateTime<Utc>,
}

impl Transformation {
    pub fn new(name: String, transformation_type: TransformationType) -> Self {
        Self {
            name,
            transformation_type,
            template: None,
            alphabet: Some(Alphabet::Alphanumeric),
            tweak_source: None,
            masking_char: Some('*'),
            masking_pattern: Some(MaskingPattern::Default),
            created_at: Utc::now(),
        }
    }
}

/// Transform role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRole {
    pub name: String,
    pub transformations: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Tokenization audit statistics
/// Requirement 10.5: Return usage stats without exposing original values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizationAuditStats {
    pub total_tokens: usize,
    pub transformations: Vec<TransformationStats>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationStats {
    pub transformation_name: String,
    pub token_count: usize,
    pub total_encode_operations: usize,
    pub total_decode_operations: usize,
    pub oldest_token: Option<DateTime<Utc>>,
    pub newest_token: Option<DateTime<Utc>>,
}

impl TransformRole {
    pub fn new(name: String, transformations: Vec<String>) -> Self {
        Self {
            name,
            transformations,
            created_at: Utc::now(),
        }
    }

    pub fn can_use(&self, transformation_name: &str) -> bool {
        self.transformations
            .contains(&transformation_name.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenMapping {
    token: String,
    plaintext: String,
    transformation_name: String,
    created_at: DateTime<Utc>,
    encode_count: usize,
    decode_count: usize,
    last_accessed: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct TransformationKey {
    transformation_name: String,
    key: FpeKey,
    created_at: DateTime<Utc>,
}

/// Transform secrets engine
pub struct TransformEngine {
    pool: Option<Pool>,
    transformations: Arc<RwLock<HashMap<String, Transformation>>>,
    roles: Arc<RwLock<HashMap<String, TransformRole>>>,
    keys: Arc<RwLock<HashMap<String, TransformationKey>>>,
    token_mappings: Arc<RwLock<HashMap<String, TokenMapping>>>,
    reverse_mappings: Arc<RwLock<HashMap<String, String>>>,
}

impl TransformEngine {
    pub fn new() -> Self {
        Self {
            pool: None,
            transformations: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            keys: Arc::new(RwLock::new(HashMap::new())),
            token_mappings: Arc::new(RwLock::new(HashMap::new())),
            reverse_mappings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_storage(pool: Pool) -> Self {
        Self {
            pool: Some(pool),
            transformations: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            keys: Arc::new(RwLock::new(HashMap::new())),
            token_mappings: Arc::new(RwLock::new(HashMap::new())),
            reverse_mappings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_transformation(
        &self,
        transformation: Transformation,
    ) -> Result<(), TransformError> {
        let transformations = self.transformations.read().await;
        if transformations.contains_key(&transformation.name) {
            return Err(TransformError::TransformationAlreadyExists(
                transformation.name.clone(),
            ));
        }
        drop(transformations);

        if transformation.transformation_type == TransformationType::FPE {
            let key = FpeKey::generate();
            let key_entry = TransformationKey {
                transformation_name: transformation.name.clone(),
                key,
                created_at: Utc::now(),
            };
            let mut keys = self.keys.write().await;
            keys.insert(transformation.name.clone(), key_entry);
        }

        let mut transformations = self.transformations.write().await;
        transformations.insert(transformation.name.clone(), transformation.clone());
        info!("Created transformation: {}", transformation.name);
        Ok(())
    }

    pub async fn get_transformation(&self, name: &str) -> Option<Transformation> {
        let transformations = self.transformations.read().await;
        transformations.get(name).cloned()
    }

    pub async fn list_transformations(&self) -> Vec<String> {
        let transformations = self.transformations.read().await;
        transformations.keys().cloned().collect()
    }

    pub async fn encode(
        &self,
        role_name: &str,
        transformation_name: &str,
        value: &str,
        tweak: Option<&str>,
    ) -> Result<String, TransformError> {
        let roles = self.roles.read().await;
        let role = roles
            .get(role_name)
            .ok_or_else(|| TransformError::RoleNotFound(role_name.to_string()))?;
        if !role.can_use(transformation_name) {
            return Err(TransformError::AccessDenied(
                role_name.to_string(),
                transformation_name.to_string(),
            ));
        }
        drop(roles);

        let transformations = self.transformations.read().await;
        let transformation = transformations
            .get(transformation_name)
            .ok_or_else(|| TransformError::TransformationNotFound(transformation_name.to_string()))?
            .clone();
        drop(transformations);

        match transformation.transformation_type {
            TransformationType::FPE => self.encode_fpe(&transformation, value, tweak).await,
            TransformationType::Tokenization => {
                self.encode_tokenization(&transformation, value).await
            }
            TransformationType::Masking => self.encode_masking(&transformation, value).await,
        }
    }

    pub async fn decode(
        &self,
        role_name: &str,
        transformation_name: &str,
        value: &str,
        tweak: Option<&str>,
    ) -> Result<String, TransformError> {
        let roles = self.roles.read().await;
        let role = roles
            .get(role_name)
            .ok_or_else(|| TransformError::RoleNotFound(role_name.to_string()))?;
        if !role.can_use(transformation_name) {
            return Err(TransformError::AccessDenied(
                role_name.to_string(),
                transformation_name.to_string(),
            ));
        }
        drop(roles);

        let transformations = self.transformations.read().await;
        let transformation = transformations
            .get(transformation_name)
            .ok_or_else(|| TransformError::TransformationNotFound(transformation_name.to_string()))?
            .clone();
        drop(transformations);

        match transformation.transformation_type {
            TransformationType::FPE => self.decode_fpe(&transformation, value, tweak).await,
            TransformationType::Tokenization => self.decode_tokenization(value).await,
            TransformationType::Masking => Err(TransformError::DecodeFailed(
                "Masking is irreversible".to_string(),
            )),
        }
    }

    async fn encode_fpe(
        &self,
        transformation: &Transformation,
        value: &str,
        tweak: Option<&str>,
    ) -> Result<String, TransformError> {
        let alphabet = transformation
            .alphabet
            .as_ref()
            .ok_or_else(|| TransformError::InvalidAlphabet("No alphabet specified".to_string()))?;

        let keys = self.keys.read().await;
        let key_entry = keys
            .get(&transformation.name)
            .ok_or_else(|| TransformError::EncodeFailed("FPE key not found".to_string()))?;

        let engine = FpeEngine::new(key_entry.key.clone(), alphabet.to_fpe_alphabet())?;
        let tweak_bytes = tweak
            .map(|t| t.as_bytes())
            .or_else(|| transformation.tweak_source.as_ref().map(|s| s.as_bytes()))
            .unwrap_or(b"");

        let ciphertext = engine
            .encrypt(value, tweak_bytes)
            .map_err(|e| TransformError::EncodeFailed(e.to_string()))?;

        debug!(
            "FPE encoded value for transformation: {}",
            transformation.name
        );
        Ok(ciphertext)
    }

    async fn decode_fpe(
        &self,
        transformation: &Transformation,
        value: &str,
        tweak: Option<&str>,
    ) -> Result<String, TransformError> {
        let alphabet = transformation
            .alphabet
            .as_ref()
            .ok_or_else(|| TransformError::InvalidAlphabet("No alphabet specified".to_string()))?;

        let keys = self.keys.read().await;
        let key_entry = keys
            .get(&transformation.name)
            .ok_or_else(|| TransformError::DecodeFailed("FPE key not found".to_string()))?;

        let engine = FpeEngine::new(key_entry.key.clone(), alphabet.to_fpe_alphabet())?;
        let tweak_bytes = tweak
            .map(|t| t.as_bytes())
            .or_else(|| transformation.tweak_source.as_ref().map(|s| s.as_bytes()))
            .unwrap_or(b"");

        let plaintext = engine
            .decrypt(value, tweak_bytes)
            .map_err(|e| TransformError::DecodeFailed(e.to_string()))?;

        debug!(
            "FPE decoded value for transformation: {}",
            transformation.name
        );
        Ok(plaintext)
    }

    async fn encode_tokenization(
        &self,
        transformation: &Transformation,
        value: &str,
    ) -> Result<String, TransformError> {
        let reverse_key = format!("{}:{}", value, transformation.name);

        {
            let reverse = self.reverse_mappings.read().await;
            if let Some(existing_token) = reverse.get(&reverse_key) {
                debug!("Reusing existing token");
                return Ok(existing_token.clone());
            }
        }

        let token = format!("tok_{}", Uuid::new_v4().simple());
        let now = Utc::now();
        let mapping = TokenMapping {
            token: token.clone(),
            plaintext: value.to_string(),
            transformation_name: transformation.name.clone(),
            created_at: now,
            encode_count: 1,
            decode_count: 0,
            last_accessed: now,
        };

        let mut token_mappings = self.token_mappings.write().await;
        let mut reverse = self.reverse_mappings.write().await;
        token_mappings.insert(token.clone(), mapping);
        reverse.insert(reverse_key, token.clone());

        debug!(
            "Created new token for transformation: {}",
            transformation.name
        );
        Ok(token)
    }

    async fn decode_tokenization(&self, token: &str) -> Result<String, TransformError> {
        let mut mappings = self.token_mappings.write().await;
        let mapping = mappings
            .get_mut(token)
            .ok_or_else(|| TransformError::DecodeFailed("Token not found".to_string()))?;

        // Update statistics
        mapping.decode_count += 1;
        mapping.last_accessed = Utc::now();

        debug!("Decoded token");
        Ok(mapping.plaintext.clone())
    }

    async fn encode_masking(
        &self,
        transformation: &Transformation,
        value: &str,
    ) -> Result<String, TransformError> {
        let mask_char = transformation.masking_char.unwrap_or('*');
        let pattern = transformation
            .masking_pattern
            .as_ref()
            .unwrap_or(&MaskingPattern::Default);

        match pattern {
            MaskingPattern::CreditCard => self.mask_credit_card(value, mask_char),
            MaskingPattern::Email => self.mask_email(value, mask_char),
            MaskingPattern::Phone => self.mask_phone(value, mask_char),
            MaskingPattern::Custom => {
                if let Some(ref template) = transformation.template {
                    self.apply_template(value, template, mask_char)
                } else {
                    Err(TransformError::InvalidTemplate(
                        "Custom masking requires template".to_string(),
                    ))
                }
            }
            MaskingPattern::Default => {
                if let Some(ref template) = transformation.template {
                    self.apply_template(value, template, mask_char)
                } else {
                    let len = value.len();
                    if len <= 4 {
                        Ok(mask_char.to_string().repeat(len))
                    } else {
                        let masked = mask_char.to_string().repeat(len - 4);
                        let visible = &value[len - 4..];
                        Ok(format!("{}{}", masked, visible))
                    }
                }
            }
        }
    }

    /// Mask credit card number (show last 4 digits)
    /// Format: ****-****-****-1234
    fn mask_credit_card(&self, value: &str, mask_char: char) -> Result<String, TransformError> {
        let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() < 13 || digits.len() > 19 {
            return Err(TransformError::InvalidTemplate(
                "Invalid credit card number length".to_string(),
            ));
        }

        let len = digits.len();
        let masked_count = len - 4;
        let masked = mask_char.to_string().repeat(masked_count);
        let visible = &digits[masked_count..];

        // Format with dashes for readability
        if len == 16 {
            Ok(format!(
                "{}-{}-{}-{}",
                &masked[0..4],
                &masked[4..8],
                &masked[8..12],
                visible
            ))
        } else {
            Ok(format!("{}{}", masked, visible))
        }
    }

    /// Mask email address (show first char and domain)
    /// Format: j***@domain.com
    fn mask_email(&self, value: &str, mask_char: char) -> Result<String, TransformError> {
        if let Some(at_pos) = value.find('@') {
            let local = &value[..at_pos];
            let domain = &value[at_pos..];

            if local.is_empty() {
                return Err(TransformError::InvalidTemplate(
                    "Invalid email format".to_string(),
                ));
            }

            let first_char = local.chars().next().unwrap();
            let masked_len = local.len().saturating_sub(1);
            let masked = mask_char.to_string().repeat(masked_len);

            Ok(format!("{}{}{}", first_char, masked, domain))
        } else {
            Err(TransformError::InvalidTemplate(
                "Invalid email format: missing @".to_string(),
            ))
        }
    }

    /// Mask phone number (show last 4 digits)
    /// Format: ***-***-1234
    fn mask_phone(&self, value: &str, mask_char: char) -> Result<String, TransformError> {
        let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() < 7 {
            return Err(TransformError::InvalidTemplate(
                "Invalid phone number length".to_string(),
            ));
        }

        let len = digits.len();
        let masked_count = len - 4;
        let masked = mask_char.to_string().repeat(masked_count);
        let visible = &digits[masked_count..];

        // Format with dashes for US phone numbers
        if len == 10 {
            Ok(format!("{}-{}-{}", &masked[0..3], &masked[3..6], visible))
        } else {
            Ok(format!("{}{}", masked, visible))
        }
    }

    fn apply_template(
        &self,
        value: &str,
        template: &str,
        mask_char: char,
    ) -> Result<String, TransformError> {
        let value_chars: Vec<char> = value.chars().filter(|c| c.is_alphanumeric()).collect();
        let mut value_idx = 0;

        let result: String = template
            .chars()
            .map(|t| match t {
                '#' => {
                    if value_idx < value_chars.len() {
                        let c = value_chars[value_idx];
                        value_idx += 1;
                        c
                    } else {
                        mask_char
                    }
                }
                '*' => {
                    value_idx += 1;
                    mask_char
                }
                _ => t,
            })
            .collect();

        Ok(result)
    }

    pub async fn create_role(&self, role: TransformRole) -> Result<(), TransformError> {
        let roles = self.roles.read().await;
        if roles.contains_key(&role.name) {
            return Err(TransformError::RoleAlreadyExists(role.name.clone()));
        }
        drop(roles);

        let mut roles = self.roles.write().await;
        roles.insert(role.name.clone(), role.clone());
        info!("Created transform role: {}", role.name);
        Ok(())
    }

    pub async fn get_role(&self, name: &str) -> Option<TransformRole> {
        let roles = self.roles.read().await;
        roles.get(name).cloned()
    }

    pub async fn list_roles(&self) -> Vec<String> {
        let roles = self.roles.read().await;
        roles.keys().cloned().collect()
    }

    /// Batch encode multiple values with consistent mapping
    /// Requirement 10.4: Process multiple values with consistent mapping
    pub async fn batch_encode(
        &self,
        role_name: &str,
        transformation_name: &str,
        values: &[String],
        tweak: Option<&str>,
    ) -> Result<Vec<String>, TransformError> {
        // Verify role access once
        let roles = self.roles.read().await;
        let role = roles
            .get(role_name)
            .ok_or_else(|| TransformError::RoleNotFound(role_name.to_string()))?;
        if !role.can_use(transformation_name) {
            return Err(TransformError::AccessDenied(
                role_name.to_string(),
                transformation_name.to_string(),
            ));
        }
        drop(roles);

        let transformations = self.transformations.read().await;
        let transformation = transformations
            .get(transformation_name)
            .ok_or_else(|| TransformError::TransformationNotFound(transformation_name.to_string()))?
            .clone();
        drop(transformations);

        // Process all values
        let mut results = Vec::with_capacity(values.len());
        for value in values {
            let encoded = match transformation.transformation_type {
                TransformationType::FPE => self.encode_fpe(&transformation, value, tweak).await?,
                TransformationType::Tokenization => {
                    self.encode_tokenization(&transformation, value).await?
                }
                TransformationType::Masking => self.encode_masking(&transformation, value).await?,
            };
            results.push(encoded);
        }

        debug!(
            "Batch encoded {} values for transformation: {}",
            values.len(),
            transformation_name
        );
        Ok(results)
    }

    /// Batch decode multiple values
    pub async fn batch_decode(
        &self,
        role_name: &str,
        transformation_name: &str,
        values: &[String],
        tweak: Option<&str>,
    ) -> Result<Vec<String>, TransformError> {
        // Verify role access once
        let roles = self.roles.read().await;
        let role = roles
            .get(role_name)
            .ok_or_else(|| TransformError::RoleNotFound(role_name.to_string()))?;
        if !role.can_use(transformation_name) {
            return Err(TransformError::AccessDenied(
                role_name.to_string(),
                transformation_name.to_string(),
            ));
        }
        drop(roles);

        let transformations = self.transformations.read().await;
        let transformation = transformations
            .get(transformation_name)
            .ok_or_else(|| TransformError::TransformationNotFound(transformation_name.to_string()))?
            .clone();
        drop(transformations);

        // Process all values
        let mut results = Vec::with_capacity(values.len());
        for value in values {
            let decoded = match transformation.transformation_type {
                TransformationType::FPE => self.decode_fpe(&transformation, value, tweak).await?,
                TransformationType::Tokenization => self.decode_tokenization(value).await?,
                TransformationType::Masking => {
                    return Err(TransformError::DecodeFailed(
                        "Masking is irreversible".to_string(),
                    ));
                }
            };
            results.push(decoded);
        }

        debug!(
            "Batch decoded {} values for transformation: {}",
            values.len(),
            transformation_name
        );
        Ok(results)
    }

    /// Get tokenization audit statistics
    /// Requirement 10.5: Return usage stats without exposing original values
    pub async fn get_audit_statistics(&self) -> Result<TokenizationAuditStats, TransformError> {
        let mappings = self.token_mappings.read().await;

        // Group by transformation
        let mut stats_map: HashMap<String, Vec<&TokenMapping>> = HashMap::new();
        for mapping in mappings.values() {
            stats_map
                .entry(mapping.transformation_name.clone())
                .or_insert_with(Vec::new)
                .push(mapping);
        }

        let mut transformations = Vec::new();
        for (transformation_name, tokens) in stats_map {
            let token_count = tokens.len();
            let total_encode_operations: usize = tokens.iter().map(|t| t.encode_count).sum();
            let total_decode_operations: usize = tokens.iter().map(|t| t.decode_count).sum();

            let oldest_token = tokens.iter().map(|t| t.created_at).min();
            let newest_token = tokens.iter().map(|t| t.created_at).max();

            transformations.push(TransformationStats {
                transformation_name,
                token_count,
                total_encode_operations,
                total_decode_operations,
                oldest_token,
                newest_token,
            });
        }

        Ok(TokenizationAuditStats {
            total_tokens: mappings.len(),
            transformations,
            generated_at: Utc::now(),
        })
    }
}

impl Default for TransformEngine {
    fn default() -> Self {
        Self::new()
    }
}
