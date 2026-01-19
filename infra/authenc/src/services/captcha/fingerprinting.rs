//! Browser Fingerprinting Module
//!
//! Privacy-compliant browser fingerprinting for bot detection

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use super::error::CaptchaError;
use super::types::*;

/// Extended browser fingerprint with additional detection capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedBrowserFingerprint {
    /// Basic fingerprint data
    pub basic: BrowserFingerprint,

    /// Advanced fingerprinting data
    pub advanced: AdvancedFingerprint,

    /// Privacy-compliant hashed identifiers
    pub hashed_identifiers: HashedIdentifiers,

    /// Fingerprint analysis results
    pub analysis: FingerprintAnalysis,
}

/// Advanced fingerprinting data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedFingerprint {
    /// Number of logical processors available
    pub hardware_concurrency: Option<u32>,
    /// Amount of device memory in gigabytes
    pub device_memory: Option<f64>,
    /// Maximum number of touch points supported
    pub max_touch_points: Option<u32>,
    /// Color depth of the screen
    pub color_depth: Option<u32>,
    /// Pixel depth of the screen
    pub pixel_depth: Option<u32>,
    /// Screen orientation (portrait/landscape)
    pub screen_orientation: Option<String>,
    /// Network connection type
    pub connection_type: Option<String>,
    /// Network connection downlink speed
    pub connection_downlink: Option<f64>,
    /// Network connection round-trip time
    pub connection_rtt: Option<u32>,
    /// Audio context fingerprint hash
    pub audio_context_fingerprint: Option<String>,
    /// List of available fonts
    pub available_fonts: Vec<String>,
    /// WebRTC fingerprint hash
    pub webrtc_fingerprint: Option<String>,
    /// Battery charge level (0.0 to 1.0)
    pub battery_level: Option<f64>,
    /// Whether battery is currently charging
    pub battery_charging: Option<bool>,
    /// Number of media input/output devices
    pub media_devices_count: Option<u32>,
    /// Performance timing fingerprint hash
    pub performance_fingerprint: Option<String>,
}

/// Privacy-compliant hashed identifiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashedIdentifiers {
    /// Hashed combination of stable identifiers
    pub device_hash: String,

    /// Hashed browser configuration
    pub browser_hash: String,

    /// Hashed hardware configuration
    pub hardware_hash: String,

    /// Session-specific hash
    pub session_hash: String,
}

/// Fingerprint analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintAnalysis {
    /// Uniqueness score (0.0 - 1.0)
    pub uniqueness_score: f64,
    /// Consistency score with previous fingerprints
    pub consistency_score: f64,
    /// Automation detection indicators
    pub automation_indicators: Vec<AutomationIndicator>,
    /// Privacy risk assessment
    pub privacy_risk: PrivacyRisk,
    /// Bot probability based on fingerprint
    pub bot_probability: f64,
}

/// Automation detection indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationIndicator {
    /// Type of automation indicator detected
    pub indicator_type: String,
    /// Description of the indicator
    pub description: String,
    /// Confidence level in the detection (0.0 to 1.0)
    pub confidence: f64,
    /// Severity level of the indicator
    pub severity: IndicatorSeverity,
}

/// Indicator severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorSeverity {
    Low,
    /// Medium severity automation indicator
    Medium,
    /// High severity automation indicator
    High,
    /// Critical severity automation indicator
    Critical,
}

/// Privacy risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Low severity automation indicator
pub struct PrivacyRisk {
    /// Overall privacy risk level
    pub risk_level: PrivacyRiskLevel,
    /// Resistance to tracking (0.0 to 1.0)
    pub tracking_resistance: f64,
    /// Anonymity score (0.0 to 1.0)
    pub anonymity_score: f64,
}

/// Privacy risk levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyRiskLevel {
    Minimal,
    /// Low privacy risk
    Low,
    /// Medium privacy risk
    Medium,
    /// High privacy risk
    High,
}

/// Fingerprinting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Minimal privacy risk
pub struct FingerprintingConfig {
    /// Enable advanced fingerprinting techniques
    pub enable_advanced_fingerprinting: bool,

    /// Enable canvas fingerprinting
    pub enable_canvas_fingerprinting: bool,

    /// Enable WebGL fingerprinting
    pub enable_webgl_fingerprinting: bool,

    /// Enable audio context fingerprinting
    pub enable_audio_fingerprinting: bool,

    /// Enable font detection
    pub enable_font_detection: bool,

    /// Enable WebRTC fingerprinting
    pub enable_webrtc_fingerprinting: bool,

    /// Privacy compliance mode
    pub privacy_compliance_mode: bool,

    /// Data retention period (days)
    pub data_retention_days: u32,

    /// Hash salt for privacy protection
    pub hash_salt: String,
}

impl Default for FingerprintingConfig {
    fn default() -> Self {
        Self {
            enable_advanced_fingerprinting: true,
            enable_canvas_fingerprinting: true,
            enable_webgl_fingerprinting: true,
            enable_audio_fingerprinting: false, // Disabled by default for privacy
            enable_font_detection: true,
            enable_webrtc_fingerprinting: false, // Disabled by default for privacy
            privacy_compliance_mode: true,
            data_retention_days: 30,
            hash_salt: "captcha_fingerprint_salt".to_string(),
        }
    }
}

/// Browser fingerprinting engine trait
#[async_trait]
pub trait FingerprintingEngine: Send + Sync {
    async fn analyze_basic_fingerprint(
        &self,
        fingerprint: &BrowserFingerprint,
    ) -> Result<FingerprintAnalysis, CaptchaError>;

    /// Create extended fingerprint from client data
    async fn create_extended_fingerprint(
        &self,
        client_data: &HashMap<String, serde_json::Value>,
    ) -> Result<ExtendedBrowserFingerprint, CaptchaError>;

    /// Detect automation indicators
    async fn detect_automation_indicators(
        &self,
        fingerprint: &ExtendedBrowserFingerprint,
    ) -> Result<Vec<AutomationIndicator>, CaptchaError>;

    /// Calculate fingerprint uniqueness
    async fn calculate_uniqueness(
        &self,
        fingerprint: &ExtendedBrowserFingerprint,
    ) -> Result<f64, CaptchaError>;

    /// Compare fingerprints for consistency
    async fn compare_fingerprints(
        &self,
        fp1: &ExtendedBrowserFingerprint,
        fp2: &ExtendedBrowserFingerprint,
    ) -> Result<f64, CaptchaError>;

    /// Generate privacy-compliant hashes
    async fn generate_hashed_identifiers(
        &self,
        fingerprint: &ExtendedBrowserFingerprint,
    ) -> Result<HashedIdentifiers, CaptchaError>;

    /// Assess privacy risk
    async fn assess_privacy_risk(
        &self,
        fingerprint: &ExtendedBrowserFingerprint,
    ) -> Result<PrivacyRisk, CaptchaError>;
}

/// Default fingerprinting engine implementation
    /// Analyze basic browser fingerprint
pub struct DefaultFingerprintingEngine {
    config: FingerprintingConfig,

    // In-memory storage for fingerprint comparison (would be database in production)
    fingerprint_database: HashMap<String, ExtendedBrowserFingerprint>,
}

impl DefaultFingerprintingEngine {
    /// Create a new fingerprinting engine
    pub fn new() -> Self {
        Self {
            config: FingerprintingConfig::default(),
            fingerprint_database: HashMap::new(),
        }
    }

    /// Create a new fingerprinting engine with custom configuration
    pub fn with_config(config: FingerprintingConfig) -> Self {
        Self {
            config,
            fingerprint_database: HashMap::new(),
        }
    }

    /// Hash a string with salt for privacy protection
    fn hash_with_salt(&self, input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hasher.update(self.config.hash_salt.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Detect headless browser indicators
    fn detect_headless_indicators(
        &self,
        fingerprint: &ExtendedBrowserFingerprint,
    ) -> Vec<AutomationIndicator> {
        let mut indicators = Vec::new();

        // Check user agent for headless indicators
        let user_agent = &fingerprint.basic.user_agent.to_lowercase();
        let headless_patterns = ["headlesschrome", "phantomjs", "htmlunit", "chrome-headless"];

        for pattern in &headless_patterns {
            if user_agent.contains(pattern) {
                indicators.push(AutomationIndicator {
                    indicator_type: "headless_browser".to_string(),
                    description: format!("Headless browser pattern detected: {}", pattern),
                    confidence: 0.9,
                    severity: IndicatorSeverity::High,
                });
            }
        }

        // Check for missing expected properties
        if fingerprint.basic.plugins.is_empty() {
            indicators.push(AutomationIndicator {
                indicator_type: "no_plugins".to_string(),
                description: "No browser plugins detected".to_string(),
                confidence: 0.6,
                severity: IndicatorSeverity::Medium,
            });
        }

        // Check for suspicious hardware values
        if let Some(concurrency) = fingerprint.advanced.hardware_concurrency {
            if concurrency == 1 || concurrency > 32 {
                indicators.push(AutomationIndicator {
                    indicator_type: "suspicious_hardware".to_string(),
                    description: format!("Unusual hardware concurrency: {}", concurrency),
                    confidence: 0.5,
                    severity: IndicatorSeverity::Low,
                });
            }
        }

        indicators
    }

    /// Detect automation framework indicators
    fn detect_automation_frameworks(
        &self,
        fingerprint: &ExtendedBrowserFingerprint,
    ) -> Vec<AutomationIndicator> {
        let mut indicators = Vec::new();

        let user_agent = &fingerprint.basic.user_agent.to_lowercase();
        let automation_patterns = [
            ("selenium", "Selenium WebDriver"),
            ("webdriver", "WebDriver automation"),
            ("puppeteer", "Puppeteer automation"),
            ("playwright", "Playwright automation"),
            ("chromedriver", "ChromeDriver automation"),
        ];

        for (pattern, description) in &automation_patterns {
            if user_agent.contains(pattern) {
                indicators.push(AutomationIndicator {
                    indicator_type: "automation_framework".to_string(),
                    description: description.to_string(),
                    confidence: 0.95,
                    severity: IndicatorSeverity::Critical,
                });
            }
        }

        indicators
    }

    /// Detect virtual machine indicators
    fn detect_vm_indicators(
        &self,
        fingerprint: &ExtendedBrowserFingerprint,
    ) -> Vec<AutomationIndicator> {
        let mut indicators = Vec::new();

        // Check for VM-specific hardware configurations
        if let Some(memory) = fingerprint.advanced.device_memory {
            // Common VM memory configurations
            let vm_memory_sizes = [1.0, 2.0, 4.0, 8.0];
            if vm_memory_sizes.contains(&memory) {
                indicators.push(AutomationIndicator {
                    indicator_type: "vm_memory_pattern".to_string(),
                    description: format!("Common VM memory size detected: {}GB", memory),
                    confidence: 0.3,
                    severity: IndicatorSeverity::Low,
                });
            }
        }

        // Check screen resolution for common VM patterns
        let resolution = &fingerprint.basic.screen_resolution;
        let vm_resolutions = ["1024x768", "1280x1024", "1366x768", "1920x1080"];

        if vm_resolutions.contains(&resolution.as_str()) {
            indicators.push(AutomationIndicator {
                indicator_type: "vm_resolution_pattern".to_string(),
                description: format!("Common VM resolution detected: {}", resolution),
                confidence: 0.2,
                severity: IndicatorSeverity::Low,
            });
        }

        indicators
    }

    /// Calculate entropy of fingerprint components
    fn calculate_fingerprint_entropy(&self, fingerprint: &ExtendedBrowserFingerprint) -> f64 {
        let mut entropy_components = Vec::new();

        // Basic components
        entropy_components.push(fingerprint.basic.user_agent.len() as f64);
        entropy_components.push(fingerprint.basic.plugins.len() as f64);
        entropy_components.push(fingerprint.basic.language.len() as f64);

        // Advanced components
        if let Some(concurrency) = fingerprint.advanced.hardware_concurrency {
            entropy_components.push(concurrency as f64);
        }

        if let Some(memory) = fingerprint.advanced.device_memory {
            entropy_components.push(memory);
        }

        entropy_components.push(fingerprint.advanced.available_fonts.len() as f64);

        // Calculate normalized entropy
        if entropy_components.is_empty() {
            return 0.0;
        }

        let sum: f64 = entropy_components.iter().sum();
        let mean = sum / entropy_components.len() as f64;

        if mean == 0.0 {
            return 0.0;
        }

        let variance = entropy_components
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / entropy_components.len() as f64;

        // Normalize to 0-1 range
        (variance.sqrt() / mean).min(1.0)
    }
}

impl Default for DefaultFingerprintingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FingerprintingEngine for DefaultFingerprintingEngine {
    async fn analyze_basic_fingerprint(
        &self,
        fingerprint: &BrowserFingerprint,
    ) -> Result<FingerprintAnalysis, CaptchaError> {
        // Create extended fingerprint from basic data
        let mut client_data = HashMap::new();
        client_data.insert(
            "user_agent".to_string(),
            serde_json::Value::String(fingerprint.user_agent.clone()),
        );
        client_data.insert(
            "screen_resolution".to_string(),
            serde_json::Value::String(fingerprint.screen_resolution.clone()),
        );
        client_data.insert(
            "timezone".to_string(),
            serde_json::Value::String(fingerprint.timezone.clone()),
        );
        client_data.insert(
            "language".to_string(),
            serde_json::Value::String(fingerprint.language.clone()),
        );
        client_data.insert(
            "plugins".to_string(),
            serde_json::Value::Array(
                fingerprint
                    .plugins
                    .iter()
                    .map(|p| serde_json::Value::String(p.clone()))
                    .collect(),
            ),
        );

        let extended_fp = self.create_extended_fingerprint(&client_data).await?;
        Ok(extended_fp.analysis)
    }

    async fn create_extended_fingerprint(
        &self,
        client_data: &HashMap<String, serde_json::Value>,
    ) -> Result<ExtendedBrowserFingerprint, CaptchaError> {
        // Extract basic fingerprint data
        let basic = BrowserFingerprint {
            user_agent: client_data
                .get("user_agent")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            screen_resolution: client_data
                .get("screen_resolution")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            timezone: client_data
                .get("timezone")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            language: client_data
                .get("language")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            plugins: client_data
                .get("plugins")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            canvas_fingerprint: client_data
                .get("canvas_fingerprint")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            webgl_fingerprint: client_data
                .get("webgl_fingerprint")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        // Extract advanced fingerprint data
        let advanced = AdvancedFingerprint {
            hardware_concurrency: client_data
                .get("hardware_concurrency")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            device_memory: client_data.get("device_memory").and_then(|v| v.as_f64()),
            max_touch_points: client_data
                .get("max_touch_points")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            color_depth: client_data
                .get("color_depth")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            pixel_depth: client_data
                .get("pixel_depth")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            screen_orientation: client_data
                .get("screen_orientation")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            connection_type: client_data
                .get("connection_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            connection_downlink: client_data
                .get("connection_downlink")
                .and_then(|v| v.as_f64()),
            connection_rtt: client_data
                .get("connection_rtt")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            audio_context_fingerprint: client_data
                .get("audio_context_fingerprint")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            available_fonts: client_data
                .get("available_fonts")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            webrtc_fingerprint: client_data
                .get("webrtc_fingerprint")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            battery_level: client_data.get("battery_level").and_then(|v| v.as_f64()),
            battery_charging: client_data
                .get("battery_charging")
                .and_then(|v| v.as_bool()),
            media_devices_count: client_data
                .get("media_devices_count")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            performance_fingerprint: client_data
                .get("performance_fingerprint")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        // Create temporary extended fingerprint for analysis
        let mut extended_fp = ExtendedBrowserFingerprint {
            basic,
            advanced,
            hashed_identifiers: HashedIdentifiers {
                device_hash: String::new(),
                browser_hash: String::new(),
                hardware_hash: String::new(),
                session_hash: String::new(),
            },
            analysis: FingerprintAnalysis {
                uniqueness_score: 0.0,
                consistency_score: 0.0,
                automation_indicators: Vec::new(),
                privacy_risk: PrivacyRisk {
                    risk_level: PrivacyRiskLevel::Minimal,
                    tracking_resistance: 1.0,
                    anonymity_score: 1.0,
                },
                bot_probability: 0.0,
            },
        };

        // Generate hashed identifiers
        extended_fp.hashed_identifiers = self.generate_hashed_identifiers(&extended_fp).await?;

        // Detect automation indicators
        extended_fp.analysis.automation_indicators =
            self.detect_automation_indicators(&extended_fp).await?;

        // Calculate uniqueness
        extended_fp.analysis.uniqueness_score = self.calculate_uniqueness(&extended_fp).await?;

        // Assess privacy risk
        extended_fp.analysis.privacy_risk = self.assess_privacy_risk(&extended_fp).await?;

        // Calculate bot probability based on indicators
        let indicator_score = extended_fp
            .analysis
            .automation_indicators
            .iter()
            .map(|indicator| match indicator.severity {
                IndicatorSeverity::Critical => indicator.confidence * 0.4,
                IndicatorSeverity::High => indicator.confidence * 0.3,
                IndicatorSeverity::Medium => indicator.confidence * 0.2,
                IndicatorSeverity::Low => indicator.confidence * 0.1,
            })
            .sum::<f64>();

        extended_fp.analysis.bot_probability = indicator_score.min(1.0);

        Ok(extended_fp)
    }

    async fn detect_automation_indicators(
        &self,
        fingerprint: &ExtendedBrowserFingerprint,
    ) -> Result<Vec<AutomationIndicator>, CaptchaError> {
        let mut indicators = Vec::new();

        // Detect headless browsers
        indicators.extend(self.detect_headless_indicators(fingerprint));

        // Detect automation frameworks
        indicators.extend(self.detect_automation_frameworks(fingerprint));

        // Detect virtual machines
        indicators.extend(self.detect_vm_indicators(fingerprint));

        Ok(indicators)
    }

    async fn calculate_uniqueness(
        &self,
        fingerprint: &ExtendedBrowserFingerprint,
    ) -> Result<f64, CaptchaError> {
        // Calculate entropy-based uniqueness score
        let entropy = self.calculate_fingerprint_entropy(fingerprint);

        // Factor in the number of unique components
        let mut unique_components = 0;

        if !fingerprint.basic.user_agent.is_empty() {
            unique_components += 1;
        }
        if !fingerprint.basic.screen_resolution.is_empty() {
            unique_components += 1;
        }
        if !fingerprint.basic.timezone.is_empty() {
            unique_components += 1;
        }
        if !fingerprint.basic.language.is_empty() {
            unique_components += 1;
        }
        if !fingerprint.basic.plugins.is_empty() {
            unique_components += 1;
        }
        if fingerprint.basic.canvas_fingerprint.is_some() {
            unique_components += 1;
        }
        if fingerprint.basic.webgl_fingerprint.is_some() {
            unique_components += 1;
        }

        // Advanced components
        if fingerprint.advanced.hardware_concurrency.is_some() {
            unique_components += 1;
        }
        if fingerprint.advanced.device_memory.is_some() {
            unique_components += 1;
        }
        if !fingerprint.advanced.available_fonts.is_empty() {
            unique_components += 1;
        }

        let component_score = unique_components as f64 / 10.0; // Normalize to 0-1

        // Combine entropy and component scores
        let uniqueness_score = (entropy * 0.6 + component_score * 0.4).min(1.0);

        Ok(uniqueness_score)
    }

    async fn compare_fingerprints(
        &self,
        fp1: &ExtendedBrowserFingerprint,
        fp2: &ExtendedBrowserFingerprint,
    ) -> Result<f64, CaptchaError> {
        let mut similarity_score = 0.0;
        let mut total_components = 0;

        // Compare basic components
        if fp1.basic.user_agent == fp2.basic.user_agent {
            similarity_score += 1.0;
        }
        total_components += 1;

        if fp1.basic.screen_resolution == fp2.basic.screen_resolution {
            similarity_score += 1.0;
        }
        total_components += 1;

        if fp1.basic.timezone == fp2.basic.timezone {
            similarity_score += 1.0;
        }
        total_components += 1;

        if fp1.basic.language == fp2.basic.language {
            similarity_score += 1.0;
        }
        total_components += 1;

        // Compare advanced components
        if fp1.advanced.hardware_concurrency == fp2.advanced.hardware_concurrency {
            similarity_score += 1.0;
        }
        total_components += 1;

        if fp1.advanced.device_memory == fp2.advanced.device_memory {
            similarity_score += 1.0;
        }
        total_components += 1;

        // Compare hashed identifiers
        if fp1.hashed_identifiers.device_hash == fp2.hashed_identifiers.device_hash {
            similarity_score += 2.0;
        }
        total_components += 2;

        let consistency_score = if total_components > 0 {
            similarity_score / total_components as f64
        } else {
            0.0
        };

        Ok(consistency_score)
    }

    async fn generate_hashed_identifiers(
        &self,
        fingerprint: &ExtendedBrowserFingerprint,
    ) -> Result<HashedIdentifiers, CaptchaError> {
        // Generate device hash from stable hardware characteristics
        let device_components = format!(
            "{}|{}|{}|{}",
            fingerprint.basic.screen_resolution,
            fingerprint.advanced.hardware_concurrency.unwrap_or(0),
            fingerprint.advanced.device_memory.unwrap_or(0.0),
            fingerprint.advanced.max_touch_points.unwrap_or(0)
        );
        let device_hash = self.hash_with_salt(&device_components);

        // Generate browser hash from browser-specific characteristics
        let browser_components = format!(
            "{}|{}|{}",
            fingerprint.basic.user_agent,
            fingerprint.basic.language,
            fingerprint.basic.plugins.join(",")
        );
        let browser_hash = self.hash_with_salt(&browser_components);

        // Generate hardware hash
        let hardware_components = format!(
            "{}|{}|{}",
            fingerprint.advanced.color_depth.unwrap_or(0),
            fingerprint.advanced.pixel_depth.unwrap_or(0),
            fingerprint.advanced.available_fonts.join(",")
        );
        let hardware_hash = self.hash_with_salt(&hardware_components);

        // Generate session hash (includes timestamp for uniqueness)
        let session_components = format!(
            "{}|{}|{}",
            device_hash,
            browser_hash,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        let session_hash = self.hash_with_salt(&session_components);

        Ok(HashedIdentifiers {
            device_hash,
            browser_hash,
            hardware_hash,
            session_hash,
        })
    }

    async fn assess_privacy_risk(
        &self,
        fingerprint: &ExtendedBrowserFingerprint,
    ) -> Result<PrivacyRisk, CaptchaError> {
        let mut tracking_factors = 0;
        let mut total_factors = 0;

        // Assess tracking potential of each component
        if !fingerprint.basic.user_agent.is_empty() {
            tracking_factors += 1;
        }
        total_factors += 1;

        if !fingerprint.basic.screen_resolution.is_empty() {
            tracking_factors += 1;
        }
        total_factors += 1;

        if fingerprint.basic.canvas_fingerprint.is_some() {
            tracking_factors += 2;
        } // Higher tracking potential
        total_factors += 2;

        if fingerprint.basic.webgl_fingerprint.is_some() {
            tracking_factors += 2;
        }
        total_factors += 2;

        if !fingerprint.advanced.available_fonts.is_empty() {
            tracking_factors += 1;
        }
        total_factors += 1;

        let tracking_resistance = if total_factors > 0 {
            1.0 - (tracking_factors as f64 / total_factors as f64)
        } else {
            1.0
        };

        // Calculate anonymity score based on uniqueness (lower uniqueness = higher anonymity)
        let anonymity_score = 1.0 - fingerprint.analysis.uniqueness_score;

        // Determine risk level
        let risk_level = if tracking_resistance < 0.3 {
            PrivacyRiskLevel::High
        } else if tracking_resistance < 0.5 {
            PrivacyRiskLevel::Medium
        } else if tracking_resistance < 0.8 {
            PrivacyRiskLevel::Low
        } else {
            PrivacyRiskLevel::Minimal
        };

        Ok(PrivacyRisk {
            risk_level,
            tracking_resistance,
            anonymity_score,
        })
    }
}
