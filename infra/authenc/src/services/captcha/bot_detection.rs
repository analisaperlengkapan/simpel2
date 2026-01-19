//! Bot Detection Algorithms
//!
//! Machine learning models and statistical analysis for bot detection

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::analyzer::*;
use super::error::CaptchaError;
use super::types::*;

/// Machine learning model types for bot detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MLModelType {
    NaiveBayes,
    /// Support Vector Machine classifier
    SVM,
    /// Random Forest ensemble classifier
    RandomForest,
    /// Neural Network deep learning model
    NeuralNetwork,
    /// Ensemble combining multiple model types
    Ensemble,
}

/// Feature vector for machine learning models
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Naive Bayes probabilistic classifier
pub struct FeatureVector {
    /// Mean mouse movement velocity
    pub mouse_velocity_mean: f64,
    /// Standard deviation of mouse velocity
    pub mouse_velocity_std: f64,
    /// Smoothness score of mouse trajectory
    pub mouse_trajectory_smoothness: f64,
    /// Ratio of pauses in mouse movement
    pub mouse_pause_ratio: f64,
    /// Number of direction changes in mouse path
    pub mouse_direction_changes: f64,
    /// Mean keystroke dwell time
    pub keystroke_dwell_mean: f64,
    /// Standard deviation of keystroke dwell time
    pub keystroke_dwell_std: f64,
    /// Mean flight time between keystrokes
    pub keystroke_flight_mean: f64,
    /// Standard deviation of flight time
    pub keystroke_flight_std: f64,
    /// Consistency of keystroke rhythm
    pub keystroke_rhythm_consistency: f64,
    /// Total interaction duration
    pub interaction_duration: f64,
    /// Frequency of pauses during interaction
    pub pause_frequency: f64,
    /// Overall rhythm score
    pub rhythm_score: f64,
    /// Uniqueness score of browser fingerprint
    pub fingerprint_uniqueness: f64,
    /// Count of automation indicators detected
    pub automation_indicator_count: f64,
    /// Number of browser plugins installed
    pub plugin_count: f64,
    /// Ratio of velocity to acceleration
    pub velocity_acceleration_ratio: f64,
    /// Consistency of typing speed
    pub typing_speed_consistency: f64,
    /// Entropy measure of behavioral patterns
    pub behavioral_entropy: f64,
}

/// Machine learning model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLModelConfig {
    /// Type of machine learning model to use
    pub model_type: MLModelType,
    /// Minimum confidence threshold for classification
    pub confidence_threshold: f64,
    /// Weights for different features in the model
    pub feature_weights: HashMap<String, f64>,
    /// Size of training dataset used
    pub training_data_size: usize,
    /// Interval for retraining the model in seconds
    pub retraining_interval: u64, // seconds
}

impl Default for MLModelConfig {
    fn default() -> Self {
        let mut feature_weights = HashMap::new();

        // Mouse movement weights
        feature_weights.insert("mouse_velocity_mean".to_string(), 0.15);
        feature_weights.insert("mouse_velocity_std".to_string(), 0.12);
        feature_weights.insert("mouse_trajectory_smoothness".to_string(), 0.10);
        feature_weights.insert("mouse_pause_ratio".to_string(), 0.08);
        feature_weights.insert("mouse_direction_changes".to_string(), 0.08);

        // Keystroke weights
        feature_weights.insert("keystroke_dwell_mean".to_string(), 0.12);
        feature_weights.insert("keystroke_dwell_std".to_string(), 0.10);
        feature_weights.insert("keystroke_rhythm_consistency".to_string(), 0.10);

        // Browser fingerprint weights
        feature_weights.insert("automation_indicator_count".to_string(), 0.15);

        Self {
            model_type: MLModelType::Ensemble,
            confidence_threshold: 0.7,
            feature_weights,
            training_data_size: 10000,
            retraining_interval: 86400, // 24 hours
        }
    }
}

/// Statistical anomaly detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionConfig {
    /// Z-score threshold for outlier detection
    pub z_score_threshold: f64,
    /// Minimum samples required for statistical analysis
    pub min_samples: usize,
    /// Window size for moving average calculations
    pub window_size: usize,
    /// Percentile thresholds for outlier detection
    pub outlier_percentiles: (f64, f64), // (lower, upper)
}

impl Default for AnomalyDetectionConfig {
    fn default() -> Self {
        Self {
            z_score_threshold: 2.5,
            min_samples: 10,
            window_size: 50,
            outlier_percentiles: (5.0, 95.0),
        }
    }
}

/// Real-time risk scoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScoringConfig {
    /// Base risk score for new sessions
    pub base_risk_score: f64,
    /// Risk score decay rate per minute
    pub decay_rate: f64,
    /// Maximum risk score
    pub max_risk_score: f64,
    /// Risk score thresholds for different actions
    pub thresholds: RiskThresholds,
}

/// Configuration for risk threshold levels in bot detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskThresholds {
    /// Threshold for low risk classification
    pub low_risk: f64,
    /// Threshold for medium risk classification
    pub medium_risk: f64,
    /// Threshold for high risk classification
    pub high_risk: f64,
    /// Threshold for critical risk classification
    pub critical_risk: f64,
}

impl Default for RiskScoringConfig {
    fn default() -> Self {
        Self {
            base_risk_score: 0.1,
            decay_rate: 0.05,
            max_risk_score: 1.0,
            thresholds: RiskThresholds {
                low_risk: 0.3,
                medium_risk: 0.5,
                high_risk: 0.7,
                critical_risk: 0.9,
            },
        }
    }
}

/// Bot detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotDetectionResult {
    /// Whether the behavior is classified as bot
    pub is_bot: bool,
    /// Confidence level in the classification (0.0 to 1.0)
    pub confidence: f64,
    /// Overall risk score (0.0 to 1.0)
    pub risk_score: f64,
    /// Final behavior classification
    pub classification: BehaviorClassification,
    /// Predictions from individual models
    pub model_predictions: HashMap<String, f64>,
    /// Anomaly scores for different features
    pub anomaly_scores: HashMap<String, f64>,
    /// Feature importance scores
    pub feature_importance: HashMap<String, f64>,
}

/// Bot detection engine trait
#[async_trait]
pub trait BotDetectionEngine: Send + Sync {
    async fn extract_features(
        &self,
        analysis: &BehavioralAnalysisResult,
    ) -> Result<FeatureVector, CaptchaError>;

    /// Predict bot probability using machine learning models
    async fn predict_bot_probability(&self, features: &FeatureVector) -> Result<f64, CaptchaError>;

    /// Detect anomalies in behavioral patterns
    async fn detect_anomalies(
        &self,
        features: &FeatureVector,
    ) -> Result<HashMap<String, f64>, CaptchaError>;

    /// Calculate real-time risk score
    async fn calculate_risk_score(
        &self,
        session_id: &str,
        current_score: f64,
        new_evidence: f64,
    ) -> Result<f64, CaptchaError>;

    /// Perform comprehensive bot detection
    async fn detect_bot(
        &self,
        analysis: &BehavioralAnalysisResult,
        session_id: &str,
    ) -> Result<BotDetectionResult, CaptchaError>;

    /// Update model with new training data
    async fn update_model(
        &mut self,
        features: &FeatureVector,
        label: bool,
    ) -> Result<(), CaptchaError>;
}

/// Default bot detection engine implementation
    /// Extract features from behavioral analysis
pub struct DefaultBotDetectionEngine {
    ml_config: MLModelConfig,
    anomaly_config: AnomalyDetectionConfig,
    risk_config: RiskScoringConfig,

    // Simple in-memory storage for baseline statistics
    feature_baselines: HashMap<String, (f64, f64)>, // (mean, std_dev)
    session_risk_scores: HashMap<String, (f64, u64)>, // (score, last_update_timestamp)
}

impl DefaultBotDetectionEngine {
    /// Create a new bot detection engine with default configuration
    pub fn new() -> Self {
        Self {
            ml_config: MLModelConfig::default(),
            anomaly_config: AnomalyDetectionConfig::default(),
            risk_config: RiskScoringConfig::default(),
            feature_baselines: HashMap::new(),
            session_risk_scores: HashMap::new(),
        }
    }

    /// Create a new bot detection engine with custom configuration
    pub fn with_config(
        ml_config: MLModelConfig,
        anomaly_config: AnomalyDetectionConfig,
        risk_config: RiskScoringConfig,
    ) -> Self {
        Self {
            ml_config,
            anomaly_config,
            risk_config,
            feature_baselines: HashMap::new(),
            session_risk_scores: HashMap::new(),
        }
    }

    /// Initialize baseline statistics (would be loaded from database in production)
    pub fn initialize_baselines(&mut self) {
        // Initialize with typical human behavior baselines
        self.feature_baselines
            .insert("mouse_velocity_mean".to_string(), (150.0, 50.0));
        self.feature_baselines
            .insert("mouse_velocity_std".to_string(), (75.0, 25.0));
        self.feature_baselines
            .insert("mouse_trajectory_smoothness".to_string(), (0.7, 0.15));
        self.feature_baselines
            .insert("keystroke_dwell_mean".to_string(), (100.0, 30.0));
        self.feature_baselines
            .insert("keystroke_dwell_std".to_string(), (40.0, 15.0));
        self.feature_baselines
            .insert("keystroke_rhythm_consistency".to_string(), (0.6, 0.2));
        self.feature_baselines
            .insert("automation_indicator_count".to_string(), (0.0, 0.5));
    }

    /// Calculate z-score for anomaly detection
    fn calculate_z_score(&self, value: f64, feature_name: &str) -> f64 {
        if let Some((mean, std_dev)) = self.feature_baselines.get(feature_name) {
            if *std_dev > 0.0 {
                (value - mean) / std_dev
            } else {
                0.0
            }
        } else {
            0.0 // No baseline available
        }
    }

    /// Naive Bayes classifier implementation
    fn naive_bayes_predict(&self, features: &FeatureVector) -> f64 {
        let mut bot_score = 0.0;
        let mut feature_count = 0;

        // Mouse velocity analysis
        if features.mouse_velocity_mean > 1000.0 || features.mouse_velocity_mean < 10.0 {
            bot_score += 0.3;
        }
        feature_count += 1;

        // Mouse trajectory smoothness (too perfect = bot)
        if features.mouse_trajectory_smoothness > 0.95 {
            bot_score += 0.25;
        }
        feature_count += 1;

        // Keystroke consistency (too consistent = bot)
        if features.keystroke_rhythm_consistency > 0.9 {
            bot_score += 0.2;
        }
        feature_count += 1;

        // Automation indicators
        if features.automation_indicator_count > 0.0 {
            bot_score += features.automation_indicator_count * 0.4;
        }
        feature_count += 1;

        // Behavioral entropy (too low = bot)
        if features.behavioral_entropy < 0.3 {
            bot_score += 0.15;
        }
        feature_count += 1;

        bot_score / feature_count as f64
    }

    /// Support Vector Machine classifier (simplified implementation)
    fn svm_predict(&self, features: &FeatureVector) -> f64 {
        // Simplified linear SVM with hand-tuned weights
        let weights = vec![
            ("mouse_velocity_mean", 0.1),
            ("mouse_velocity_std", -0.15),
            ("mouse_trajectory_smoothness", 0.2),
            ("keystroke_dwell_std", -0.1),
            ("keystroke_rhythm_consistency", 0.25),
            ("automation_indicator_count", 0.4),
            ("behavioral_entropy", -0.2),
        ];

        let mut score = 0.0;
        for (feature_name, weight) in weights {
            let feature_value = match feature_name {
                "mouse_velocity_mean" => features.mouse_velocity_mean / 1000.0, // Normalize
                "mouse_velocity_std" => features.mouse_velocity_std / 100.0,
                "mouse_trajectory_smoothness" => features.mouse_trajectory_smoothness,
                "keystroke_dwell_std" => features.keystroke_dwell_std / 100.0,
                "keystroke_rhythm_consistency" => features.keystroke_rhythm_consistency,
                "automation_indicator_count" => features.automation_indicator_count,
                "behavioral_entropy" => features.behavioral_entropy,
                _ => 0.0,
            };
            score += weight * feature_value;
        }

        // Apply sigmoid function to get probability
        1.0 / (1.0 + (-score).exp())
    }

    /// Random Forest classifier (simplified implementation)
    fn random_forest_predict(&self, features: &FeatureVector) -> f64 {
        // Simulate multiple decision trees with different thresholds
        let mut tree_predictions = Vec::new();

        // Tree 1: Focus on mouse behavior
        let tree1_score =
            if features.mouse_velocity_mean > 500.0 && features.mouse_trajectory_smoothness > 0.9 {
                0.8
            } else if features.mouse_velocity_std < 20.0 {
                0.6
            } else {
                0.2
            };
        tree_predictions.push(tree1_score);

        // Tree 2: Focus on keystroke behavior
        let tree2_score = if features.keystroke_rhythm_consistency > 0.85
            && features.keystroke_dwell_std < 15.0
        {
            0.9
        } else if features.keystroke_dwell_mean < 50.0 || features.keystroke_dwell_mean > 300.0 {
            0.7
        } else {
            0.3
        };
        tree_predictions.push(tree2_score);

        // Tree 3: Focus on automation indicators
        let tree3_score = if features.automation_indicator_count > 1.0 {
            0.95
        } else if features.fingerprint_uniqueness < 0.3 {
            0.6
        } else {
            0.1
        };
        tree_predictions.push(tree3_score);

        // Tree 4: Focus on behavioral entropy
        let tree4_score = if features.behavioral_entropy < 0.2 {
            0.8
        } else if features.behavioral_entropy > 0.8 {
            0.1
        } else {
            0.4
        };
        tree_predictions.push(tree4_score);

        // Average predictions
        tree_predictions.iter().sum::<f64>() / tree_predictions.len() as f64
    }

    /// Ensemble prediction combining multiple models
    fn ensemble_predict(&self, features: &FeatureVector) -> f64 {
        let naive_bayes = self.naive_bayes_predict(features);
        let svm = self.svm_predict(features);
        let random_forest = self.random_forest_predict(features);

        // Weighted ensemble
        let weights = [0.3, 0.4, 0.3]; // [naive_bayes, svm, random_forest]
        weights[0] * naive_bayes + weights[1] * svm + weights[2] * random_forest
    }

    /// Calculate behavioral entropy
    fn calculate_behavioral_entropy(&self, features: &FeatureVector) -> f64 {
        // Simple entropy calculation based on feature variance
        let variances = vec![
            features.mouse_velocity_std,
            features.keystroke_dwell_std,
            features.keystroke_flight_std,
        ];

        let total_variance = variances.iter().sum::<f64>();
        if total_variance > 0.0 {
            // Normalize to 0-1 range
            (total_variance / 1000.0).min(1.0)
        } else {
            0.0
        }
    }
}

impl Default for DefaultBotDetectionEngine {
    fn default() -> Self {
        let mut engine = Self::new();
        engine.initialize_baselines();
        engine
    }
}

#[async_trait]
impl BotDetectionEngine for DefaultBotDetectionEngine {
    async fn extract_features(
        &self,
        analysis: &BehavioralAnalysisResult,
    ) -> Result<FeatureVector, CaptchaError> {
        // Extract mouse features
        let (
            mouse_velocity_mean,
            mouse_velocity_std,
            mouse_trajectory_smoothness,
            mouse_pause_ratio,
            mouse_direction_changes,
        ) = if let Some(ref mouse) = analysis.mouse_analysis {
            (
                mouse.average_velocity,
                mouse.velocity_variance.sqrt(),
                mouse.trajectory_smoothness,
                mouse.pause_count as f64
                    / (mouse.pause_count + mouse.direction_changes).max(1) as f64,
                mouse.direction_changes as f64,
            )
        } else {
            (0.0, 0.0, 1.0, 0.0, 0.0)
        };

        // Extract keystroke features
        let (
            keystroke_dwell_mean,
            keystroke_dwell_std,
            keystroke_flight_mean,
            keystroke_flight_std,
            keystroke_rhythm_consistency,
        ) = if let Some(ref keystroke) = analysis.keystroke_analysis {
            (
                keystroke.average_dwell_time,
                keystroke.dwell_time_variance.sqrt(),
                keystroke.average_flight_time,
                keystroke.flight_time_variance.sqrt(),
                keystroke.typing_rhythm_consistency,
            )
        } else {
            (0.0, 0.0, 0.0, 0.0, 0.0)
        };

        // Extract timing features
        let interaction_duration = analysis.timing_analysis.interaction_duration as f64;
        let pause_frequency = analysis.timing_analysis.pause_frequency;
        let rhythm_score = analysis.timing_analysis.rhythm_score;

        // Extract fingerprint features
        let fingerprint_uniqueness = analysis.fingerprint_analysis.uniqueness_score;
        let automation_indicator_count =
            analysis.fingerprint_analysis.automation_indicators.len() as f64;
        let plugin_count = 0.0; // Would be extracted from fingerprint data

        // Calculate derived features
        let velocity_acceleration_ratio = if mouse_velocity_std > 0.0 {
            mouse_velocity_mean / mouse_velocity_std
        } else {
            0.0
        };

        let typing_speed_consistency = if keystroke_dwell_std > 0.0 {
            keystroke_dwell_mean / keystroke_dwell_std
        } else {
            0.0
        };

        let mut features = FeatureVector {
            mouse_velocity_mean,
            mouse_velocity_std,
            mouse_trajectory_smoothness,
            mouse_pause_ratio,
            mouse_direction_changes,
            keystroke_dwell_mean,
            keystroke_dwell_std,
            keystroke_flight_mean,
            keystroke_flight_std,
            keystroke_rhythm_consistency,
            interaction_duration,
            pause_frequency,
            rhythm_score,
            fingerprint_uniqueness,
            automation_indicator_count,
            plugin_count,
            velocity_acceleration_ratio,
            typing_speed_consistency,
            behavioral_entropy: 0.0, // Will be calculated
        };

        // Calculate behavioral entropy
        features.behavioral_entropy = self.calculate_behavioral_entropy(&features);

        Ok(features)
    }

    async fn predict_bot_probability(&self, features: &FeatureVector) -> Result<f64, CaptchaError> {
        let prediction = match self.ml_config.model_type {
            MLModelType::NaiveBayes => self.naive_bayes_predict(features),
            MLModelType::SVM => self.svm_predict(features),
            MLModelType::RandomForest => self.random_forest_predict(features),
            MLModelType::NeuralNetwork => {
                // Placeholder for neural network - would use actual ML library
                self.ensemble_predict(features)
            }
            MLModelType::Ensemble => self.ensemble_predict(features),
        };

        Ok(prediction.clamp(0.0, 1.0))
    }

    async fn detect_anomalies(
        &self,
        features: &FeatureVector,
    ) -> Result<HashMap<String, f64>, CaptchaError> {
        let mut anomaly_scores = HashMap::new();

        // Calculate z-scores for key features
        let feature_values = vec![
            ("mouse_velocity_mean", features.mouse_velocity_mean),
            ("mouse_velocity_std", features.mouse_velocity_std),
            (
                "mouse_trajectory_smoothness",
                features.mouse_trajectory_smoothness,
            ),
            ("keystroke_dwell_mean", features.keystroke_dwell_mean),
            ("keystroke_dwell_std", features.keystroke_dwell_std),
            (
                "keystroke_rhythm_consistency",
                features.keystroke_rhythm_consistency,
            ),
            (
                "automation_indicator_count",
                features.automation_indicator_count,
            ),
        ];

        for (feature_name, value) in feature_values {
            let z_score = self.calculate_z_score(value, feature_name);
            let anomaly_score = if z_score.abs() > self.anomaly_config.z_score_threshold {
                (z_score.abs() - self.anomaly_config.z_score_threshold) / 5.0 // Normalize
            } else {
                0.0
            };
            anomaly_scores.insert(feature_name.to_string(), anomaly_score.min(1.0));
        }

        Ok(anomaly_scores)
    }

    async fn calculate_risk_score(
        &self,
        session_id: &str,
        current_score: f64,
        new_evidence: f64,
    ) -> Result<f64, CaptchaError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Apply time decay to existing risk score
        let decayed_score =
            if let Some((last_score, last_update)) = self.session_risk_scores.get(session_id) {
                let time_diff = (now - last_update) as f64 / 60.0; // minutes
                let decay_factor = (-self.risk_config.decay_rate * time_diff).exp();
                last_score * decay_factor
            } else {
                self.risk_config.base_risk_score
            };

        // Combine with new evidence
        let updated_score =
            (decayed_score + new_evidence * 0.3).min(self.risk_config.max_risk_score);

        Ok(updated_score)
    }

    async fn detect_bot(
        &self,
        analysis: &BehavioralAnalysisResult,
        session_id: &str,
    ) -> Result<BotDetectionResult, CaptchaError> {
        // Extract features
        let features = self.extract_features(analysis).await?;

        // Get ML predictions
        let bot_probability = self.predict_bot_probability(&features).await?;

        // Detect anomalies
        let anomaly_scores = self.detect_anomalies(&features).await?;

        // Calculate overall anomaly score
        let overall_anomaly_score = if !anomaly_scores.is_empty() {
            anomaly_scores.values().sum::<f64>() / anomaly_scores.len() as f64
        } else {
            0.0
        };

        // Update risk score
        let risk_score = self
            .calculate_risk_score(session_id, 0.0, bot_probability)
            .await?;

        // Determine classification
        let is_bot = bot_probability >= self.ml_config.confidence_threshold;
        let classification = if is_bot {
            BehaviorClassification::Bot
        } else if bot_probability >= 0.5 {
            BehaviorClassification::Suspicious
        } else {
            BehaviorClassification::Human
        };

        // Calculate feature importance (simplified)
        let mut feature_importance = HashMap::new();
        for (feature_name, weight) in &self.ml_config.feature_weights {
            feature_importance.insert(feature_name.clone(), *weight);
        }

        // Create model predictions map
        let mut model_predictions = HashMap::new();
        model_predictions.insert("ensemble".to_string(), bot_probability);
        model_predictions.insert("anomaly_detection".to_string(), overall_anomaly_score);

        Ok(BotDetectionResult {
            is_bot,
            confidence: analysis.confidence,
            risk_score,
            classification,
            model_predictions,
            anomaly_scores,
            feature_importance,
        })
    }

    async fn update_model(
        &mut self,
        features: &FeatureVector,
        label: bool,
    ) -> Result<(), CaptchaError> {
        // In a real implementation, this would update the ML models with new training data
        // For now, we'll update the baseline statistics

        let feature_values = vec![
            ("mouse_velocity_mean", features.mouse_velocity_mean),
            ("mouse_velocity_std", features.mouse_velocity_std),
            ("keystroke_dwell_mean", features.keystroke_dwell_mean),
            ("keystroke_dwell_std", features.keystroke_dwell_std),
        ];

        for (feature_name, value) in feature_values {
            if let Some((mean, std_dev)) = self.feature_baselines.get_mut(feature_name) {
                // Simple online update (exponential moving average)
                let alpha = 0.1; // Learning rate
                *mean = (1.0 - alpha) * *mean + alpha * value;

                // Update standard deviation (simplified)
                let variance = std_dev.powi(2);
                let new_variance = (1.0 - alpha) * variance + alpha * (value - *mean).powi(2);
                *std_dev = new_variance.sqrt();
            }
        }

        Ok(())
    }
}
