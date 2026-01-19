//! Behavioral Analysis Module
//!
//! Implements behavioral analysis for bot detection using mouse movement,
//! keystroke dynamics, timing patterns, and browser fingerprinting

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::error::CaptchaError;
use super::types::*;

/// Behavioral analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralAnalysisConfig {
    /// Minimum number of mouse events required for analysis
    pub min_mouse_events: usize,
    /// Minimum number of keystroke events required for analysis
    pub min_keystroke_events: usize,
    /// Threshold for bot detection (0.0 - 1.0)
    pub bot_detection_threshold: f64,
    /// Threshold for suspicious behavior (0.0 - 1.0)
    pub suspicious_threshold: f64,
    /// Maximum allowed velocity for human-like mouse movement
    pub max_human_velocity: f64,
    /// Minimum time between keystrokes for human typing
    pub min_human_keystroke_interval: u64,
    /// Maximum time between keystrokes for human typing
    pub max_human_keystroke_interval: u64,
}

impl Default for BehavioralAnalysisConfig {
    fn default() -> Self {
        Self {
            min_mouse_events: 5,
            min_keystroke_events: 3,
            bot_detection_threshold: 0.8,
            suspicious_threshold: 0.6,
            max_human_velocity: 2000.0,         // pixels per second
            min_human_keystroke_interval: 50,   // milliseconds
            max_human_keystroke_interval: 2000, // milliseconds
        }
    }
}

/// Mouse movement analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseAnalysisResult {
    /// Average velocity of mouse movements in pixels per second
    pub average_velocity: f64,
    /// Variance in mouse movement velocity
    pub velocity_variance: f64,
    /// Smoothness score of mouse trajectory (0.0 to 1.0)
    pub trajectory_smoothness: f64,
    /// Number of pauses detected in mouse movement
    pub pause_count: usize,
    /// Number of direction changes in mouse path
    pub direction_changes: usize,
    /// Probability that this is bot behavior (0.0 to 1.0)
    pub bot_probability: f64,
}

/// Keystroke dynamics analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeAnalysisResult {
    /// Average time keys are held down in milliseconds
    pub average_dwell_time: f64,
    /// Variance in dwell time
    pub dwell_time_variance: f64,
    /// Average time between releasing one key and pressing next
    pub average_flight_time: f64,
    /// Variance in flight time
    pub flight_time_variance: f64,
    /// Consistency score of typing rhythm (0.0 to 1.0)
    pub typing_rhythm_consistency: f64,
    /// Probability that this is bot behavior (0.0 to 1.0)
    pub bot_probability: f64,
}

/// Timing pattern analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingAnalysisResult {
    /// Total duration of user interaction in milliseconds
    pub interaction_duration: u64,
    /// Frequency of pauses during interaction
    pub pause_frequency: f64,
    /// Rhythm consistency score (0.0 to 1.0)
    pub rhythm_score: f64,
    /// Probability that this is bot behavior (0.0 to 1.0)
    pub bot_probability: f64,
}

/// Browser fingerprint analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintAnalysisResult {
    /// Score indicating how unique this fingerprint is (0.0 to 1.0)
    pub uniqueness_score: f64,
    /// Score indicating fingerprint consistency over time (0.0 to 1.0)
    pub consistency_score: f64,
    /// List of detected automation indicators
    pub automation_indicators: Vec<String>,
    /// Probability that this is bot behavior (0.0 to 1.0)
    pub bot_probability: f64,
}

/// Comprehensive behavioral analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralAnalysisResult {
    /// Results from mouse movement analysis
    pub mouse_analysis: Option<MouseAnalysisResult>,
    /// Results from keystroke dynamics analysis
    pub keystroke_analysis: Option<KeystrokeAnalysisResult>,
    /// Results from timing pattern analysis
    pub timing_analysis: TimingAnalysisResult,
    /// Results from browser fingerprint analysis
    pub fingerprint_analysis: FingerprintAnalysisResult,
    /// Overall risk score combining all analyses (0.0 to 1.0)
    pub overall_risk_score: f64,
    /// Final classification of behavior
    pub classification: BehaviorClassification,
    /// Confidence level in the classification (0.0 to 1.0)
    pub confidence: f64,
}

/// Behavioral analyzer trait
#[async_trait]
pub trait BehavioralAnalyzerTrait: Send + Sync {
    async fn analyze_mouse_behavior(
        &self,
        events: &[MouseEvent],
    ) -> Result<MouseAnalysisResult, CaptchaError>;

    /// Analyze keystroke dynamics
    async fn analyze_keystroke_behavior(
        &self,
        events: &[KeystrokeEvent],
    ) -> Result<KeystrokeAnalysisResult, CaptchaError>;

    /// Analyze timing patterns
    async fn analyze_timing_patterns(
        &self,
        timing: &TimingAnalysis,
    ) -> Result<TimingAnalysisResult, CaptchaError>;

    /// Analyze browser fingerprint
    async fn analyze_browser_fingerprint(
        &self,
        fingerprint: &BrowserFingerprint,
    ) -> Result<FingerprintAnalysisResult, CaptchaError>;

    /// Perform comprehensive behavioral analysis
    async fn analyze_behavior(
        &self,
        metrics: &BehavioralMetrics,
    ) -> Result<BehavioralAnalysisResult, CaptchaError>;

    /// Update behavioral metrics with new analysis
    async fn update_behavioral_metrics(
        &self,
        metrics: &mut BehavioralMetrics,
        analysis: &BehavioralAnalysisResult,
    ) -> Result<(), CaptchaError>;
}

/// Default behavioral analyzer implementation
    /// Analyze mouse movement patterns
pub struct BehavioralAnalyzer {
    config: BehavioralAnalysisConfig,
}

impl BehavioralAnalyzer {
    /// Create a new behavioral analyzer with default configuration
    pub fn new() -> Self {
        Self {
            config: BehavioralAnalysisConfig::default(),
        }
    }

    /// Create a new behavioral analyzer with custom configuration
    pub fn with_config(config: BehavioralAnalysisConfig) -> Self {
        Self { config }
    }

    /// Calculate velocity between two mouse events
    fn calculate_velocity(&self, event1: &MouseEvent, event2: &MouseEvent) -> f64 {
        let dx = event2.x - event1.x;
        let dy = event2.y - event1.y;
        let distance = (dx * dx + dy * dy).sqrt();
        let time_diff = (event2.timestamp - event1.timestamp) as f64 / 1000.0; // Convert to seconds

        if time_diff > 0.0 {
            distance / time_diff
        } else {
            0.0
        }
    }

    /// Calculate trajectory smoothness using curvature analysis
    fn calculate_trajectory_smoothness(&self, events: &[MouseEvent]) -> f64 {
        if events.len() < 3 {
            return 1.0; // Perfect smoothness for insufficient data
        }

        let mut total_curvature = 0.0;
        let mut valid_points = 0;

        for i in 1..events.len() - 1 {
            let p1 = &events[i - 1];
            let p2 = &events[i];
            let p3 = &events[i + 1];

            // Calculate vectors
            let v1x = p2.x - p1.x;
            let v1y = p2.y - p1.y;
            let v2x = p3.x - p2.x;
            let v2y = p3.y - p2.y;

            // Calculate angle between vectors
            let dot_product = v1x * v2x + v1y * v2y;
            let mag1 = (v1x * v1x + v1y * v1y).sqrt();
            let mag2 = (v2x * v2x + v2y * v2y).sqrt();

            if mag1 > 0.0 && mag2 > 0.0 {
                let cos_angle = (dot_product / (mag1 * mag2)).clamp(-1.0, 1.0);
                let angle = cos_angle.acos();
                total_curvature += angle;
                valid_points += 1;
            }
        }

        if valid_points > 0 {
            let average_curvature = total_curvature / valid_points as f64;
            // Convert to smoothness score (lower curvature = higher smoothness)
            1.0 - (average_curvature / std::f64::consts::PI).min(1.0)
        } else {
            1.0
        }
    }

    /// Count direction changes in mouse movement
    fn count_direction_changes(&self, events: &[MouseEvent]) -> usize {
        if events.len() < 3 {
            return 0;
        }

        let mut direction_changes = 0;
        let mut last_direction = None;

        for i in 1..events.len() {
            let dx = events[i].x - events[i - 1].x;
            let dy = events[i].y - events[i - 1].y;

            if dx.abs() > 1.0 || dy.abs() > 1.0 {
                let current_direction = (dx.atan2(dy) * 180.0 / std::f64::consts::PI) as i32;

                if let Some(last_dir) = last_direction {
                    let angle_diff = (current_direction as i32 - last_dir as i32).abs() as i32;
                    let normalized_diff = if angle_diff > 180 {
                        360 - angle_diff
                    } else {
                        angle_diff
                    };

                    if normalized_diff > 45 {
                        // Significant direction change
                        direction_changes += 1;
                    }
                }

                last_direction = Some(current_direction);
            }
        }

        direction_changes
    }

    /// Calculate variance of a vector of values
    fn calculate_variance(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;

        variance
    }

    /// Detect automation indicators in browser fingerprint
    fn detect_automation_indicators(&self, fingerprint: &BrowserFingerprint) -> Vec<String> {
        let mut indicators = Vec::new();

        // Check for headless browser indicators
        if fingerprint.user_agent.contains("HeadlessChrome")
            || fingerprint.user_agent.contains("PhantomJS")
            || fingerprint.user_agent.contains("Selenium")
        {
            indicators.push("headless_browser".to_string());
        }

        // Check for automation frameworks
        if fingerprint.user_agent.contains("WebDriver")
            || fingerprint.user_agent.contains("Puppeteer")
        {
            indicators.push("automation_framework".to_string());
        }

        // Check for suspicious plugin combinations
        if fingerprint.plugins.is_empty() {
            indicators.push("no_plugins".to_string());
        }

        // Check for common bot user agents
        let bot_patterns = ["bot", "crawler", "spider", "scraper"];
        for pattern in &bot_patterns {
            if fingerprint.user_agent.to_lowercase().contains(pattern) {
                indicators.push(format!("bot_pattern_{}", pattern));
                break;
            }
        }

        indicators
    }
}

impl Default for BehavioralAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BehavioralAnalyzerTrait for BehavioralAnalyzer {
    async fn analyze_mouse_behavior(
        &self,
        events: &[MouseEvent],
    ) -> Result<MouseAnalysisResult, CaptchaError> {
        if events.len() < self.config.min_mouse_events {
            return Ok(MouseAnalysisResult {
                average_velocity: 0.0,
                velocity_variance: 0.0,
                trajectory_smoothness: 1.0,
                pause_count: 0,
                direction_changes: 0,
                bot_probability: 0.5, // Neutral when insufficient data
            });
        }

        // Calculate velocities
        let mut velocities = Vec::new();
        for i in 1..events.len() {
            let velocity = self.calculate_velocity(&events[i - 1], &events[i]);
            velocities.push(velocity);
        }

        let average_velocity = velocities.iter().sum::<f64>() / velocities.len() as f64;
        let velocity_variance = self.calculate_variance(&velocities);

        // Calculate trajectory smoothness
        let trajectory_smoothness = self.calculate_trajectory_smoothness(events);

        // Count pauses (events with very low velocity)
        let pause_count = velocities.iter().filter(|&&v| v < 10.0).count();

        // Count direction changes
        let direction_changes = self.count_direction_changes(events);

        // Calculate bot probability based on multiple factors
        let mut bot_score: f32 = 0.0;

        // Very high or very low velocity indicates bot
        if average_velocity > self.config.max_human_velocity || average_velocity < 1.0 {
            bot_score += 0.3;
        }

        // Very low variance indicates robotic movement
        if velocity_variance < 100.0 {
            bot_score += 0.2;
        }

        // Perfect smoothness is suspicious
        if trajectory_smoothness > 0.95 {
            bot_score += 0.2;
        }

        // Too few or too many direction changes
        let expected_changes = events.len() / 10;
        if direction_changes < expected_changes / 2 || direction_changes > expected_changes * 3 {
            bot_score += 0.15;
        }

        // No pauses is suspicious for human behavior
        if pause_count == 0 && events.len() > 20 {
            bot_score += 0.15;
        }

        let bot_probability = bot_score.min(1.0);

        Ok(MouseAnalysisResult {
            average_velocity,
            velocity_variance,
            trajectory_smoothness,
            pause_count,
            direction_changes,
            bot_probability: bot_probability.into(),
        })
    }

    async fn analyze_keystroke_behavior(
        &self,
        events: &[KeystrokeEvent],
    ) -> Result<KeystrokeAnalysisResult, CaptchaError> {
        if events.len() < self.config.min_keystroke_events {
            return Ok(KeystrokeAnalysisResult {
                average_dwell_time: 0.0,
                dwell_time_variance: 0.0,
                average_flight_time: 0.0,
                flight_time_variance: 0.0,
                typing_rhythm_consistency: 0.0,
                bot_probability: 0.5, // Neutral when insufficient data
            });
        }

        // Calculate dwell times
        let dwell_times: Vec<f64> = events.iter().map(|e| e.dwell_time as f64).collect();
        let average_dwell_time = dwell_times.iter().sum::<f64>() / dwell_times.len() as f64;
        let dwell_time_variance = self.calculate_variance(&dwell_times);

        // Calculate flight times
        let flight_times: Vec<f64> = events
            .iter()
            .filter_map(|e| e.flight_time.map(|ft| ft as f64))
            .collect();

        let (average_flight_time, flight_time_variance) = if !flight_times.is_empty() {
            let avg = flight_times.iter().sum::<f64>() / flight_times.len() as f64;
            let var = self.calculate_variance(&flight_times);
            (avg, var)
        } else {
            (0.0, 0.0)
        };

        // Calculate typing rhythm consistency
        let mut rhythm_scores = Vec::new();
        for i in 1..events.len() {
            let interval = events[i].timestamp - events[i - 1].timestamp;
            rhythm_scores.push(interval as f64);
        }

        let rhythm_variance = self.calculate_variance(&rhythm_scores);
        let typing_rhythm_consistency = if rhythm_variance > 0.0 {
            1.0 / (1.0 + rhythm_variance / 10000.0) // Normalize variance
        } else {
            1.0
        };

        // Calculate bot probability
        let mut bot_score: f32 = 0.0;

        // Very consistent dwell times indicate automation
        if dwell_time_variance < 100.0 && average_dwell_time > 0.0 {
            bot_score += 0.3;
        }

        // Dwell times outside human range
        if average_dwell_time < self.config.min_human_keystroke_interval as f64
            || average_dwell_time > self.config.max_human_keystroke_interval as f64
        {
            bot_score += 0.2;
        }

        // Perfect rhythm consistency is suspicious
        if typing_rhythm_consistency > 0.95 {
            bot_score += 0.25;
        }

        // Very low flight time variance
        if flight_time_variance < 50.0 && !flight_times.is_empty() {
            bot_score += 0.25;
        }

        let bot_probability = bot_score.min(1.0);

        Ok(KeystrokeAnalysisResult {
            average_dwell_time,
            dwell_time_variance,
            average_flight_time,
            flight_time_variance,
            typing_rhythm_consistency,
            bot_probability: bot_probability.into(),
        })
    }

    async fn analyze_timing_patterns(
        &self,
        timing: &TimingAnalysis,
    ) -> Result<TimingAnalysisResult, CaptchaError> {
        let interaction_duration = timing.total_interaction_time;

        // Calculate pause frequency
        let pause_frequency = if interaction_duration > 0 {
            timing.pause_patterns.len() as f64 / (interaction_duration as f64 / 1000.0)
        } else {
            0.0
        };

        let rhythm_score = timing.rhythm_consistency;

        // Calculate bot probability based on timing patterns
        let mut bot_score: f32 = 0.0;

        // Very short interaction time
        if interaction_duration < 1000 {
            // Less than 1 second
            bot_score += 0.3;
        }

        // No pauses in long interactions
        if interaction_duration > 5000 && timing.pause_patterns.is_empty() {
            bot_score += 0.2;
        }

        // Perfect rhythm consistency
        if rhythm_score > 0.95 {
            bot_score += 0.25;
        }

        // Unusual pause frequency
        if pause_frequency > 5.0 || (pause_frequency == 0.0 && interaction_duration > 3000) {
            bot_score += 0.25;
        }

        let bot_probability = bot_score.min(1.0);

        Ok(TimingAnalysisResult {
            interaction_duration,
            pause_frequency,
            rhythm_score,
            bot_probability: bot_probability.into(),
        })
    }

    async fn analyze_browser_fingerprint(
        &self,
        fingerprint: &BrowserFingerprint,
    ) -> Result<FingerprintAnalysisResult, CaptchaError> {
        // Detect automation indicators
        let automation_indicators = self.detect_automation_indicators(fingerprint);

        // Calculate uniqueness score based on fingerprint components
        let mut uniqueness_factors = 0;
        if !fingerprint.user_agent.is_empty() {
            uniqueness_factors += 1;
        }
        if !fingerprint.screen_resolution.is_empty() {
            uniqueness_factors += 1;
        }
        if !fingerprint.timezone.is_empty() {
            uniqueness_factors += 1;
        }
        if !fingerprint.language.is_empty() {
            uniqueness_factors += 1;
        }
        if !fingerprint.plugins.is_empty() {
            uniqueness_factors += 1;
        }
        if fingerprint.canvas_fingerprint.is_some() {
            uniqueness_factors += 1;
        }
        if fingerprint.webgl_fingerprint.is_some() {
            uniqueness_factors += 1;
        }

        let uniqueness_score = uniqueness_factors as f64 / 7.0; // Normalize to 0-1

        // Calculate consistency score (placeholder - would need historical data)
        let consistency_score = 0.8; // Default assumption of consistency

        // Calculate bot probability
        let mut bot_score = 0.0;

        // Automation indicators strongly suggest bot
        bot_score += automation_indicators.len() as f64 * 0.3;

        // Very low uniqueness suggests generic/headless browser
        if uniqueness_score < 0.3 {
            bot_score += 0.2;
        }

        // Missing common fingerprint components
        if fingerprint.plugins.is_empty() {
            bot_score += 0.1;
        }

        if fingerprint.canvas_fingerprint.is_none() && fingerprint.webgl_fingerprint.is_none() {
            bot_score += 0.1;
        }

        let bot_probability = bot_score.min(1.0);

        Ok(FingerprintAnalysisResult {
            uniqueness_score,
            consistency_score,
            automation_indicators,
            bot_probability,
        })
    }

    async fn analyze_behavior(
        &self,
        metrics: &BehavioralMetrics,
    ) -> Result<BehavioralAnalysisResult, CaptchaError> {
        // Analyze each component
        let mouse_analysis = if !metrics.mouse_movements.is_empty() {
            Some(
                self.analyze_mouse_behavior(&metrics.mouse_movements)
                    .await?,
            )
        } else {
            None
        };

        let keystroke_analysis = if !metrics.keystroke_dynamics.is_empty() {
            Some(
                self.analyze_keystroke_behavior(&metrics.keystroke_dynamics)
                    .await?,
            )
        } else {
            None
        };

        let timing_analysis = self
            .analyze_timing_patterns(&metrics.timing_patterns)
            .await?;
        let fingerprint_analysis = self
            .analyze_browser_fingerprint(&metrics.browser_fingerprint)
            .await?;

        // Calculate overall risk score
        let mut risk_components = Vec::new();

        if let Some(ref mouse) = mouse_analysis {
            risk_components.push(mouse.bot_probability);
        }

        if let Some(ref keystroke) = keystroke_analysis {
            risk_components.push(keystroke.bot_probability);
        }

        risk_components.push(timing_analysis.bot_probability);
        risk_components.push(fingerprint_analysis.bot_probability);

        let overall_risk_score = if !risk_components.is_empty() {
            risk_components.iter().sum::<f64>() / risk_components.len() as f64
        } else {
            0.5 // Neutral when no data
        };

        // Determine classification
        let classification = if overall_risk_score >= self.config.bot_detection_threshold {
            BehaviorClassification::Bot
        } else if overall_risk_score >= self.config.suspicious_threshold {
            BehaviorClassification::Suspicious
        } else {
            BehaviorClassification::Human
        };

        // Calculate confidence based on amount of data available
        let mut confidence_factors = 0.0;
        if mouse_analysis.is_some() {
            confidence_factors += 0.3;
        }
        if keystroke_analysis.is_some() {
            confidence_factors += 0.3;
        }
        confidence_factors += 0.2; // Timing analysis always available
        confidence_factors += 0.2; // Fingerprint analysis always available

        let confidence = confidence_factors;

        Ok(BehavioralAnalysisResult {
            mouse_analysis,
            keystroke_analysis,
            timing_analysis,
            fingerprint_analysis,
            overall_risk_score,
            classification,
            confidence,
        })
    }

    async fn update_behavioral_metrics(
        &self,
        metrics: &mut BehavioralMetrics,
        analysis: &BehavioralAnalysisResult,
    ) -> Result<(), CaptchaError> {
        // Update risk score and classification
        metrics.risk_score = analysis.overall_risk_score;
        metrics.classification = analysis.classification.clone();

        // Update timing patterns if we have keystroke data
        if let Some(ref keystroke_analysis) = analysis.keystroke_analysis {
            metrics.timing_patterns.rhythm_consistency =
                keystroke_analysis.typing_rhythm_consistency;

            if let Some(typing_speed) = metrics.timing_patterns.typing_speed {
                // Update typing speed if we have keystroke data
                if keystroke_analysis.average_dwell_time > 0.0 {
                    let new_speed = 60000.0 / keystroke_analysis.average_dwell_time; // WPM approximation
                    metrics.timing_patterns.typing_speed = Some((typing_speed + new_speed) / 2.0);
                }
            } else if keystroke_analysis.average_dwell_time > 0.0 {
                metrics.timing_patterns.typing_speed =
                    Some(60000.0 / keystroke_analysis.average_dwell_time);
            }
        }

        Ok(())
    }
}
