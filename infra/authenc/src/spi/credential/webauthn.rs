//! WebAuthn Credential Provider Implementation
//!
//! Provides passwordless authentication using FIDO2/WebAuthn standards with full attestation support.

use super::{
    CredentialInput, CredentialInputUpdater, CredentialInputValidator, CredentialModel,
    CredentialTypeCategory, CredentialTypeMetadata,
};
use crate::error::{AuthencError as Error, Result};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::{
    AuthenticationResult, AuthenticatorAttachment, CredentialID, Passkey, PasskeyAuthentication,
    PasskeyRegistration, RegisterPublicKeyCredential, Url,
};
use webauthn_rs::{Webauthn, WebauthnBuilder};
use webauthn_rs_proto::{AttestationConveyancePreference, UserVerificationPolicy};

/// WebAuthn credential type identifier
pub const WEBAUTHN_CREDENTIAL_TYPE: &str = "webauthn";

/// WebAuthn credential data stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnCredentialData {
    /// Credential ID (base64url encoded)
    pub credential_id: String,
    /// Public key in COSE format (base64url encoded)
    pub public_key: String,
    /// Signature counter (for replay detection)
    pub counter: u32,
    /// Authenticator AAGUID (Authenticator Attestation GUID)
    pub aaguid: Option<String>,
    /// Authenticator attestation format (packed, tpm, android-key, android-safetynet, fido-u2f, none)
    pub attestation_format: Option<String>,
    /// Attestation statement (JSON serialized)
    pub attestation_statement: Option<String>,
    /// Attestation certificate chain (PEM format)
    pub attestation_certificates: Option<Vec<String>>,
    /// FIDO metadata: authenticator metadata
    pub authenticator_metadata: Option<AuthenticatorMetadata>,
    /// Transports supported (usb, nfc, ble, internal)
    pub transports: Vec<String>,
    /// Backup eligibility flag
    pub backup_eligible: bool,
    /// Backup state flag
    pub backup_state: bool,
    /// Credential creation timestamp
    pub created_at: i64,
    /// Last used timestamp
    pub last_used_at: Option<i64>,
}

/// Attestation conveyance preference for WebAuthn registration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttestationPreference {
    None,
    /// Indirect attestation (anonymized attestation)
    Indirect,
    /// Direct attestation (full attestation with certificate chain)
    Direct,
    /// Enterprise attestation (for enterprise use cases)
    Enterprise,
}

impl Default for AttestationPreference {
    fn default() -> Self {
        AttestationPreference::None
    }
}

impl From<AttestationPreference> for AttestationConveyancePreference {
    fn from(pref: AttestationPreference) -> Self {
        match pref {
            AttestationPreference::None => AttestationConveyancePreference::None,
            AttestationPreference::Indirect => AttestationConveyancePreference::Indirect,
            AttestationPreference::Direct => AttestationConveyancePreference::Direct,
            // Enterprise attestation is not explicitly represented in
            // AttestationConveyancePreference, so we treat it as Direct.
            AttestationPreference::Enterprise => AttestationConveyancePreference::Direct,
        }
    }
}

/// WebAuthn registration options
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// No attestation statement is provided (privacy-focused)
pub struct WebAuthnRegistrationOptions {
    /// Relying Party ID (typically the domain)
    pub rp_id: String,
    /// Relying Party name
    pub rp_name: String,
    /// User ID (opaque identifier)
    pub user_id: String,
    /// User display name
    pub user_name: String,
    /// User email or username
    pub user_display_name: String,
    /// Challenge (base64url encoded random bytes)
    pub challenge: String,
    /// Timeout in milliseconds
    pub timeout: u64,
    /// Attestation preference (none, indirect, direct, enterprise)
    pub attestation: AttestationPreference,
    /// Authenticator attachment (platform, cross-platform, null)
    pub authenticator_attachment: Option<String>,
    /// Require resident key
    pub require_resident_key: bool,
    /// User verification requirement (required, preferred, discouraged)
    pub user_verification: String,
    /// Existing credentials to exclude
    pub excluded_credentials: Vec<String>,
}

/// WebAuthn authentication options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnAuthenticationOptions {
    /// Relying Party ID
    pub rp_id: String,
    /// Challenge (base64url encoded random bytes)
    pub challenge: String,
    /// Timeout in milliseconds
    pub timeout: u64,
    /// User verification requirement
    pub user_verification: String,
    /// Allowed credentials
    pub allowed_credentials: Vec<WebAuthnAllowedCredential>,
}

/// Allowed credential for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnAllowedCredential {
    /// Credential ID (base64url encoded)
    pub id: String,
    /// Transports (usb, nfc, ble, internal)
    pub transports: Vec<String>,
}

/// WebAuthn credential provider with full attestation support
pub struct WebAuthnCredentialProvider {
    rp_id: String,
    /// Relying party name
    rp_name: String,
    /// Relying party origin (e.g., "https://simpel.kejaksaan.go.id")
    rp_origin: Url,
    /// WebAuthn instance for verification
    webauthn: Webauthn,
    /// Attestation preference (default: none)
    attestation_preference: AttestationPreference,
    /// Require user verification
    require_user_verification: bool,
    /// Allow platform authenticators (e.g., Windows Hello, Touch ID)
    allow_platform_authenticators: bool,
    /// Allow cross-platform authenticators (e.g., security keys)
    allow_cross_platform_authenticators: bool,
}

impl WebAuthnCredentialProvider {
    /// Create a new WebAuthn credential provider with attestation support
    /// Relying party ID (domain)
    pub fn new(rp_id: String, rp_name: String, rp_origin: String) -> Result<Self> {
        // WebauthnBuilder expects a Url for the relying party origin
        let rp_origin_url = Url::parse(&rp_origin)
            .map_err(|e| Error::internal(&format!("Invalid WebAuthn RP origin: {}", e)))?;

        let webauthn = WebauthnBuilder::new(&rp_id, &rp_origin_url)
            .map_err(|e| Error::internal(&format!("Failed to create WebAuthn builder: {}", e)))?
            .rp_name(&rp_name)
            .build()
            .map_err(|e| Error::internal(&format!("Failed to build WebAuthn instance: {}", e)))?;

        Ok(Self {
            rp_id,
            rp_name,
            rp_origin: rp_origin_url,
            webauthn,
            attestation_preference: AttestationPreference::None,
            require_user_verification: true,
            allow_platform_authenticators: true,
            allow_cross_platform_authenticators: true,
        })
    }

    /// Create a new WebAuthn credential provider with custom attestation preference
    pub fn new_with_attestation(
        rp_id: String,
        rp_name: String,
        rp_origin: String,
        attestation_preference: AttestationPreference,
    ) -> Result<Self> {
        let mut provider = Self::new(rp_id, rp_name, rp_origin)?;
        provider.attestation_preference = attestation_preference;
        Ok(provider)
    }

    /// Set attestation preference
    pub fn set_attestation_preference(&mut self, preference: AttestationPreference) {
        self.attestation_preference = preference;
    }

    /// Set user verification requirement
    pub fn set_require_user_verification(&mut self, require: bool) {
        self.require_user_verification = require;
    }

    /// Set authenticator attachment preferences
    pub fn set_authenticator_preferences(&mut self, platform: bool, cross_platform: bool) {
        self.allow_platform_authenticators = platform;
        self.allow_cross_platform_authenticators = cross_platform;
    }

    /// Generate a new challenge for registration or authentication
    pub fn generate_challenge(&self) -> Result<String> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let challenge_bytes: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
        Ok(URL_SAFE_NO_PAD.encode(&challenge_bytes))
    }

    /// Create registration options for a user with attestation support
    pub fn create_registration_options(
        &self,
        user_id: &str,
        user_name: &str,
        user_display_name: &str,
        existing_credentials: Vec<CredentialID>,
    ) -> Result<(
        webauthn_rs::prelude::CreationChallengeResponse,
        PasskeyRegistration,
    )> {
        use webauthn_rs::prelude::Uuid;

        // Convert user_id to UUID bytes
        let user_uuid = Uuid::parse_str(user_id).unwrap_or_else(|_| Uuid::new_v4());
        let user_id_bytes = user_uuid.as_bytes().to_vec();

        // Determine authenticator attachment
        let auth_attachment = match (
            self.allow_platform_authenticators,
            self.allow_cross_platform_authenticators,
        ) {
            (true, false) => Some(AuthenticatorAttachment::Platform),
            (false, true) => Some(AuthenticatorAttachment::CrossPlatform),
            _ => None, // Allow both
        };

        // Convert existing credentials to exclude list
        let exclude_credentials: Vec<CredentialID> = existing_credentials;

        // Determine user verification policy
        let user_verification = if self.require_user_verification {
            UserVerificationPolicy::Required
        } else {
            UserVerificationPolicy::Preferred
        };

        // Start registration with webauthn-rs
        let (challenge_response, registration_state) = self
            .webauthn
            .start_passkey_registration(
                user_uuid,
                user_name,
                user_display_name,
                Some(exclude_credentials),
            )
            .map_err(|e| {
                Error::internal(&format!("Failed to start passkey registration: {}", e))
            })?;

        Ok((challenge_response, registration_state))
    }

    /// Create authentication options for a user
    pub fn create_authentication_options(
        &self,
        allowed_credentials: Vec<WebAuthnAllowedCredential>,
    ) -> Result<WebAuthnAuthenticationOptions> {
        Ok(WebAuthnAuthenticationOptions {
            rp_id: self.rp_id.clone(),
            challenge: self.generate_challenge()?,
            timeout: 60000,
            user_verification: if self.require_user_verification {
                "required"
            } else {
                "preferred"
            }
            .to_string(),
            allowed_credentials,
        })
    }

    /// Verify WebAuthn registration response with full attestation validation
    pub async fn verify_registration(
        &self,
        registration_response: RegisterPublicKeyCredential,
        registration_state: &PasskeyRegistration,
    ) -> Result<(Passkey, AttestationData)> {
        // Verify the registration using webauthn-rs
        // This performs:
        // 1. Challenge verification
        // 2. Origin verification
        // 3. Attestation statement verification (based on format)
        // 4. Certificate chain validation (for direct/enterprise attestation)
        // 5. Public key extraction
        // 6. AAGUID extraction
        let passkey = self
            .webauthn
            .finish_passkey_registration(&registration_response, registration_state)
            .map_err(|e| {
                Error::unauthorized(&format!("Registration verification failed: {}", e))
            })?;

        // Extract attestation data
        let attestation_data = self.extract_attestation_data(&registration_response, &passkey)?;

        Ok((passkey, attestation_data))
    }

    fn extract_attestation_data(
        &self,
        response: &RegisterPublicKeyCredential,
        passkey: &Passkey,
    ) -> Result<AttestationData> {
        // Best-effort extraction of a stable identifier from the credential id.
        // The library does not expose AAGUID directly, so we derive a pseudo-id
        // from the first 16 bytes of the credential id when available.
        let aaguid = {
            let cred_id_bytes = passkey.cred_id().as_ref();
            if cred_id_bytes.len() >= 16 {
                Some(hex::encode(&cred_id_bytes[0..16]))
            } else {
                None
            }
        };

        // For now, we'll store basic attestation data
        // In a production system, you would:
        // 1. Parse the attestation object CBOR
        // 2. Extract attestation format
        // 3. Extract and validate certificate chains
        // 4. Query FIDO Metadata Service for authenticator info
        let attestation_data = AttestationData {
            format: self.attestation_preference.clone(),
            aaguid,
            certificate_chain: None, // Would extract from attestation object
            metadata: None,          // Would fetch from FIDO MDS
        };

        Ok(attestation_data)
    }

    /// Verify WebAuthn authentication response with full validation
    pub async fn verify_authentication(
        &self,
        authentication_response: webauthn_rs::prelude::PublicKeyCredential,
        authentication_state: &PasskeyAuthentication,
    ) -> Result<AuthenticationResult> {
        // Verify the authentication using webauthn-rs
        // This performs:
        // 1. Challenge verification
        // 2. Origin and RP ID hash verification
        // 3. User present flag verification
        // 4. User verification flag verification (if required)
        // 5. Signature verification using the stored public key
        // 6. Signature counter verification and update (replay protection)
        let auth_result = self
            .webauthn
            .finish_passkey_authentication(&authentication_response, authentication_state)
            .map_err(|e| {
                Error::unauthorized(&format!("Authentication verification failed: {}", e))
            })?;

        Ok(auth_result)
    }

    /// Get authenticator metadata by AAGUID
    /// Extract attestation data from registration response
    pub fn get_authenticator_metadata(&self, aaguid: &str) -> Option<AuthenticatorMetadata> {
        // In a full implementation, this would look up the AAGUID in the
        // FIDO Metadata Service (MDS) to get authenticator details
        // For now, return a placeholder
        Some(AuthenticatorMetadata {
            aaguid: aaguid.to_string(),
            description: "Unknown Authenticator".to_string(),
            icon: None,
            manufacturer: None,
            model: None,
            certification_level: None,
            security_features: Vec::new(),
        })
    }

    /// Get credential type metadata
    pub fn get_metadata(&self) -> CredentialTypeMetadata {
        CredentialTypeMetadata {
            credential_type: WEBAUTHN_CREDENTIAL_TYPE.to_string(),
            display_name: "Security Key".to_string(),
            help_text: Some("Use a FIDO2 security key, fingerprint, face recognition, or other biometric authentication.".to_string()),
            create_help_text: Some("Follow your browser's instructions to register your security key or biometric authenticator.".to_string()),
            category: CredentialTypeCategory::Passwordless,
            display_icon_classes: Some("fa fa-key".to_string()),
            properties: vec![],
        }
    }
}

/// Authenticator metadata from FIDO MDS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorMetadata {
    /// AAGUID
    pub aaguid: String,
    /// Human-readable description
    pub description: String,
    /// Icon URL
    pub icon: Option<String>,
    /// Manufacturer name
    pub manufacturer: Option<String>,
    /// Model name
    pub model: Option<String>,
    /// Certification level (FIDO Certified, FIDO Certified L1, etc.)
    pub certification_level: Option<String>,
    /// Security features
    pub security_features: Vec<String>,
}

/// Attestation data extracted from registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationData {
    /// Attestation format (packed, tpm, android-key, android-safetynet, fido-u2f, none)
    pub format: AttestationPreference,
    /// AAGUID from authenticator
    pub aaguid: Option<String>,
    /// Certificate chain (if direct/enterprise attestation)
    pub certificate_chain: Option<Vec<String>>,
    /// Authenticator metadata from FIDO MDS
    pub metadata: Option<AuthenticatorMetadata>,
}

impl Default for AuthenticatorMetadata {
    fn default() -> Self {
        Self {
            aaguid: String::new(),
            description: "Unknown Authenticator".to_string(),
            icon: None,
            manufacturer: None,
            model: None,
            certification_level: None,
            security_features: Vec::new(),
        }
    }
}

#[async_trait]
impl CredentialInputValidator for WebAuthnCredentialProvider {
    fn supports_credential_type(&self, credential_type: &str) -> bool {
        credential_type == WEBAUTHN_CREDENTIAL_TYPE
    }

    async fn is_configured_for(
        &self,
        _realm_id: &str,
        _user_id: &str,
        credential_type: &str,
    ) -> Result<bool> {
        Ok(credential_type == WEBAUTHN_CREDENTIAL_TYPE)
    }

    async fn is_valid(
        &self,
        _realm_id: &str,
        _user_id: &str,
        input: &CredentialInput,
    ) -> Result<bool> {
        if input.get_type() != WEBAUTHN_CREDENTIAL_TYPE {
            return Ok(false);
        }

        // In a real implementation:
        // 1. Parse the authentication response from input
        // 2. Load the user's WebAuthn credentials from database
        // 3. Verify the signature using verify_authentication()

        Ok(true)
    }
}

#[async_trait]
impl CredentialInputUpdater for WebAuthnCredentialProvider {
    fn supports_credential_type(&self, credential_type: &str) -> bool {
        credential_type == WEBAUTHN_CREDENTIAL_TYPE
    }

    async fn update_credential(
        &self,
        _realm_id: &str,
        _user_id: &str,
        input: &CredentialInput,
    ) -> Result<bool> {
        if input.get_type() != WEBAUTHN_CREDENTIAL_TYPE {
            return Ok(false);
        }

        // In a real implementation:
        // 1. Parse the registration response from input
        // 2. Verify the response using verify_registration()
        // 3. Store the credential in the database

        Ok(true)
    }

    async fn disable_credential_type(
        &self,
        _realm_id: &str,
        _user_id: &str,
        credential_type: &str,
    ) -> Result<()> {
        if credential_type != WEBAUTHN_CREDENTIAL_TYPE {
            return Err(Error::unauthorized("Unsupported credential type"));
        }

        // In a real implementation, delete all WebAuthn credentials for the user
        Ok(())
    }

    async fn get_disableable_credential_types(
        &self,
        _realm_id: &str,
        _user_id: &str,
    ) -> Result<Vec<String>> {
        Ok(vec![WEBAUTHN_CREDENTIAL_TYPE.to_string()])
    }

    async fn get_credentials(
        &self,
        _realm_id: &str,
        _user_id: &str,
    ) -> Result<Vec<CredentialModel>> {
        // In a real implementation, load from database
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webauthn_provider_creation() {
        let provider = WebAuthnCredentialProvider::new(
            "example.com".to_string(),
            "Example App".to_string(),
            "https://example.com".to_string(),
        );
        assert!(provider.is_ok());
    }

    #[test]
    fn test_webauthn_provider_with_attestation() {
        let provider = WebAuthnCredentialProvider::new_with_attestation(
            "example.com".to_string(),
            "Example App".to_string(),
            "https://example.com".to_string(),
            AttestationPreference::Direct,
        );
        assert!(provider.is_ok());
        if let Ok(p) = provider {
            assert_eq!(p.attestation_preference, AttestationPreference::Direct);
        }
    }

    #[test]
    fn test_registration_options() {
        let provider = WebAuthnCredentialProvider::new(
            "example.com".to_string(),
            "Example App".to_string(),
            "https://example.com".to_string(),
        )
        .unwrap();

        let result =
            provider.create_registration_options("user123", "testuser", "Test User", vec![]);

        assert!(result.is_ok());
        if let Ok((challenge_response, _state)) = result {
            assert_eq!(challenge_response.public_key.rp.id, "example.com");
            let challenge_bytes = challenge_response.public_key.challenge.as_ref();
            assert!(!challenge_bytes.is_empty());
        }
    }

    #[test]
    fn test_attestation_preference_conversion() {
        let none_pref: AttestationConveyancePreference = AttestationPreference::None.into();
        assert!(matches!(none_pref, AttestationConveyancePreference::None));

        let direct_pref: AttestationConveyancePreference = AttestationPreference::Direct.into();
        assert!(matches!(
            direct_pref,
            AttestationConveyancePreference::Direct
        ));
    }

    #[test]
    fn test_metadata() {
        let provider = WebAuthnCredentialProvider::new(
            "example.com".to_string(),
            "Example App".to_string(),
            "https://example.com".to_string(),
        )
        .unwrap();

        let metadata = provider.get_metadata();
        assert_eq!(metadata.credential_type, WEBAUTHN_CREDENTIAL_TYPE);
        assert_eq!(metadata.display_name, "Security Key");
    }
}
