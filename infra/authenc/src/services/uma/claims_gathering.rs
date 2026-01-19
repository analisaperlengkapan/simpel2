//! UMA 2.0 Claims Gathering
//!
//! Interactive flow for collecting additional authorization information
//! when initial authorization request lacks sufficient claims.

use super::{ClaimRequirement, UmaError};
use crate::error::{AuthencError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Claims gathering request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimsGatheringRequest {
    /// Permission ticket requiring claims
    pub ticket: String,
    /// Claim token format (e.g., "urn:ietf:params:oauth:token-type:jwt")
    pub claim_token_format: Option<String>,
    /// State for resuming authorization flow
    pub state: Option<String>,
}

/// Claims gathering response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimsGatheringResponse {
    /// Claims gathering endpoint URL
    pub claims_gathering_endpoint: String,
    /// Required claims
    pub required_claims: Vec<ClaimRequirement>,
    /// State to pass back when submitting claims
    pub state: String,
    /// Optional redirect user for interactive gathering
    pub redirect_uri: Option<String>,
}

/// Submitted claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmittedClaims {
    /// State from claims gathering response
    pub state: String,
    /// Collected claims
    pub claims: HashMap<String, ClaimValue>,
    /// Format of claims (default: "urn:ietf:params:oauth:token-type:jwt")
    pub format: Option<String>,
}

/// Claim value (can be various types)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClaimValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<serde_json::Value>),
    Object(HashMap<String, serde_json::Value>),
}

/// Claims gathering state
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaimsGatheringState {
    /// State ID
    id: String,
    /// Permission ticket
    ticket: String,
    /// Required claims
    required_claims: Vec<ClaimRequirement>,
    /// Subject ID
    subject_id: String,
    /// Client ID
    client_id: String,
    /// Realm ID
    realm_id: String,
    /// Expiration timestamp
    expires_at: i64,
    /// Collected claims so far
    collected_claims: HashMap<String, ClaimValue>,
}

/// Claims gathering service
pub struct ClaimsGatheringService {
    /// Base URL for claims gathering endpoint
    base_url: String,
    /// State store (in production, use Redis or database)
    state_store: HashMap<String, ClaimsGatheringState>,
}

impl ClaimsGatheringService {
    /// Create new claims gathering service
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            state_store: HashMap::new(),
        }
    }

    /// Initiate claims gathering flow
    ///
    /// Called when authorization request fails due to missing claims
    pub fn initiate_claims_gathering(
        &mut self,
        ticket: &str,
        required_claims: Vec<ClaimRequirement>,
        subject_id: &str,
        client_id: &str,
        realm_id: &str,
    ) -> Result<ClaimsGatheringResponse> {
        // Generate state
        let state_id = Uuid::new_v4().to_string();

        // Create state
        let state = ClaimsGatheringState {
            id: state_id.clone(),
            ticket: ticket.to_string(),
            required_claims: required_claims.clone(),
            subject_id: subject_id.to_string(),
            client_id: client_id.to_string(),
            realm_id: realm_id.to_string(),
            expires_at: chrono::Utc::now().timestamp() + 600, // 10 minutes
            collected_claims: HashMap::new(),
        };

        // Store state (in production, use Redis or database)
        self.state_store.insert(state_id.clone(), state);

        // Determine if we need interactive gathering (redirect)
        let needs_interactive = self.requires_interactive_gathering(&required_claims);

        let redirect_uri = if needs_interactive {
            Some(format!(
                "{}/claims/gather?state={}",
                self.base_url, state_id
            ))
        } else {
            None
        };

        Ok(ClaimsGatheringResponse {
            claims_gathering_endpoint: format!("{}/claims/submit", self.base_url),
            required_claims,
            state: state_id,
            redirect_uri,
        })
    }

    /// Submit collected claims
    ///
    /// Called by client after gathering required claims
    pub fn submit_claims(&mut self, submitted: SubmittedClaims) -> Result<ClaimsSubmissionResult> {
        // Retrieve state
        let state = self
            .state_store
            .get_mut(&submitted.state)
            .ok_or_else(|| AuthencError::validation("Invalid or expired state".to_string()))?;

        // Check if expired
        if state.expires_at < chrono::Utc::now().timestamp() {
            self.state_store.remove(&submitted.state);
            return Err(AuthencError::validation(
                "Claims gathering session expired".to_string(),
            ));
        }

        // Validate that all required claims are provided
        let missing_claims = Self::find_missing_claims(&state.required_claims, &submitted.claims);

        if !missing_claims.is_empty() {
            return Ok(ClaimsSubmissionResult::Incomplete {
                missing_claims,
                state: submitted.state,
            });
        }

        // Validate claim values
        Self::validate_claims(&state.required_claims, &submitted.claims)?;

        // Store collected claims
        state.collected_claims.extend(submitted.claims.clone());

        // Return success with collected claims
        Ok(ClaimsSubmissionResult::Complete {
            ticket: state.ticket.clone(),
            claims: state.collected_claims.clone(),
            claim_token: Self::encode_claim_token(&state.collected_claims)?,
        })
    }

    /// Get claims gathering state
    pub fn get_state(&self, state_id: &str) -> Result<&ClaimsGatheringState> {
        self.state_store.get(state_id).ok_or_else(|| {
            AuthencError::resource_not_found("Claims gathering state not found".to_string())
        })
    }

    /// Cancel claims gathering
    pub fn cancel_claims_gathering(&mut self, state_id: &str) -> Result<()> {
        self.state_store.remove(state_id).ok_or_else(|| {
            AuthencError::resource_not_found("Claims gathering state not found".to_string())
        })?;
        Ok(())
    }

    /// Check if claims gathering requires interactive flow
    fn requires_interactive_gathering(&self, claims: &[ClaimRequirement]) -> bool {
        // Interactive gathering needed if:
        // 1. Claims require user input (not programmatically available)
        // 2. Claims need multi-factor authentication
        // 3. Claims require external verification

        claims.iter().any(|claim| {
            // Check for claims that typically require interaction
            matches!(
                claim.claim_name.as_str(),
                "user_consent" | "mfa_verification" | "biometric" | "document_upload"
            )
        })
    }

    /// Find missing required claims
    fn find_missing_claims(
        required: &[ClaimRequirement],
        submitted: &HashMap<String, ClaimValue>,
    ) -> Vec<ClaimRequirement> {
        required
            .iter()
            .filter(|req| !submitted.contains_key(&req.claim_name))
            .cloned()
            .collect()
    }

    /// Validate claim values against requirements
    fn validate_claims(
        requirements: &[ClaimRequirement],
        claims: &HashMap<String, ClaimValue>,
    ) -> Result<()> {
        for req in requirements {
            if let Some(value) = claims.get(&req.claim_name) {
                // Validate against possible values if specified
                if let Some(ref possible_values) = req.possible_values {
                    let value_str = match value {
                        ClaimValue::String(s) => s.clone(),
                        ClaimValue::Number(n) => n.to_string(),
                        ClaimValue::Boolean(b) => b.to_string(),
                        _ => continue, // Skip validation for complex types
                    };

                    if !possible_values.contains(&value_str) {
                        return Err(AuthencError::validation(format!(
                            "Invalid value for claim '{}'. Expected one of: {:?}",
                            req.claim_name, possible_values
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Encode claims as claim token (JWT or other format)
    fn encode_claim_token(claims: &HashMap<String, ClaimValue>) -> Result<String> {
        // In production, this would create a JWT with the claims
        // For now, return base64-encoded JSON

        let json = serde_json::to_string(claims)
            .map_err(|e| AuthencError::internal(format!("Failed to serialize claims: {}", e)))?;

        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.encode(json.as_bytes()))
    }
}

/// Result of claims submission
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum ClaimsSubmissionResult {
    /// All required claims collected
    Complete {
        /// Permission ticket
        ticket: String,
        /// Collected claims
        claims: HashMap<String, ClaimValue>,
        /// Encoded claim token
        claim_token: String,
    },
    /// Some claims still missing
    Incomplete {
        /// Missing claims
        missing_claims: Vec<ClaimRequirement>,
        /// State for next submission
        state: String,
    },
}

/// Claims gathering flow helper
pub struct ClaimsGatheringFlow {
    service: ClaimsGatheringService,
}

impl ClaimsGatheringFlow {
/// Fungsi `new(base_url`.
    pub fn new(base_url: String) -> Self {
        Self {
            service: ClaimsGatheringService::new(base_url),
        }
    }

    /// Handle need_info error and initiate claims gathering
    pub fn handle_need_info_error(
        &mut self,
        error: &UmaError,
        subject_id: &str,
        client_id: &str,
        realm_id: &str,
    ) -> Result<ClaimsGatheringResponse> {
        let ticket = error
            .ticket
            .as_ref()
            .ok_or_else(|| AuthencError::validation("No ticket in need_info error".to_string()))?;

        let required_claims = error.required_claims.clone().ok_or_else(|| {
            AuthencError::validation("No required claims in need_info error".to_string())
        })?;

        self.service.initiate_claims_gathering(
            ticket,
            required_claims,
            subject_id,
            client_id,
            realm_id,
        )
    }

    /// Resume authorization with collected claims
    pub fn resume_authorization(&mut self, submitted: SubmittedClaims) -> Result<(String, String)> {
        match self.service.submit_claims(submitted)? {
            ClaimsSubmissionResult::Complete {
                ticket,
                claim_token,
                ..
            } => Ok((ticket, claim_token)),
            ClaimsSubmissionResult::Incomplete { missing_claims, .. } => {
                Err(AuthencError::validation(format!(
                    "Still missing claims: {:?}",
                    missing_claims
                        .iter()
                        .map(|c| &c.claim_name)
                        .collect::<Vec<_>>()
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_gathering_initiation() {
        let mut service = ClaimsGatheringService::new("https://auth.example.com".to_string());

        let required_claims = vec![ClaimRequirement {
            claim_name: "department".to_string(),
            friendly_name: Some("Department".to_string()),
            issuer: None,
            possible_values: Some(vec![
                "engineering".to_string(),
                "sales".to_string(),
                "hr".to_string(),
            ]),
        }];

        let response = service.initiate_claims_gathering(
            "ticket123",
            required_claims,
            "user456",
            "client789",
            "realm001",
        );

        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.required_claims.len(), 1);
        assert!(!response.state.is_empty());
    }

    #[test]
    fn test_claims_submission_complete() {
        let mut service = ClaimsGatheringService::new("https://auth.example.com".to_string());

        let required_claims = vec![ClaimRequirement {
            claim_name: "department".to_string(),
            friendly_name: Some("Department".to_string()),
            issuer: None,
            possible_values: Some(vec!["engineering".to_string()]),
        }];

        let response = service
            .initiate_claims_gathering(
                "ticket123",
                required_claims,
                "user456",
                "client789",
                "realm001",
            )
            .unwrap();

        let mut claims = HashMap::new();
        claims.insert(
            "department".to_string(),
            ClaimValue::String("engineering".to_string()),
        );

        let submitted = SubmittedClaims {
            state: response.state,
            claims,
            format: None,
        };

        let result = service.submit_claims(submitted);
        assert!(result.is_ok());

        match result.unwrap() {
            ClaimsSubmissionResult::Complete { ticket, .. } => {
                assert_eq!(ticket, "ticket123");
            }
            ClaimsSubmissionResult::Incomplete { .. } => {
                panic!("Expected complete result");
            }
        }
    }

    #[test]
    fn test_claims_submission_incomplete() {
        let mut service = ClaimsGatheringService::new("https://auth.example.com".to_string());

        let required_claims = vec![
            ClaimRequirement {
                claim_name: "department".to_string(),
                friendly_name: Some("Department".to_string()),
                issuer: None,
                possible_values: None,
            },
            ClaimRequirement {
                claim_name: "clearance_level".to_string(),
                friendly_name: Some("Clearance Level".to_string()),
                issuer: None,
                possible_values: None,
            },
        ];

        let response = service
            .initiate_claims_gathering(
                "ticket123",
                required_claims,
                "user456",
                "client789",
                "realm001",
            )
            .unwrap();

        let mut claims = HashMap::new();
        claims.insert(
            "department".to_string(),
            ClaimValue::String("engineering".to_string()),
        );
        // Missing clearance_level

        let submitted = SubmittedClaims {
            state: response.state,
            claims,
            format: None,
        };

        let result = service.submit_claims(submitted);
        assert!(result.is_ok());

        match result.unwrap() {
            ClaimsSubmissionResult::Incomplete { missing_claims, .. } => {
                assert_eq!(missing_claims.len(), 1);
                assert_eq!(missing_claims[0].claim_name, "clearance_level");
            }
            ClaimsSubmissionResult::Complete { .. } => {
                panic!("Expected incomplete result");
            }
        }
    }
}
