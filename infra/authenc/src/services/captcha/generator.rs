//! Challenge Generator
//!
//! Generates various types of CAPTCHA challenges with cryptographic security

use super::adaptive_difficulty::*;
use super::audio_challenges::*;
use super::error::CaptchaError;
use super::image_challenges::*;
use super::types::*;
use async_trait::async_trait;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;

/// Visual challenge data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualChallenge {
    /// Type of visual challenge
    pub challenge_type: String,
    /// Challenge data (base64 encoded image or similar)
    pub data: String,
    /// Expected answer to the challenge
    pub answer: String,
    /// Multiple choice options if applicable
    pub options: Option<Vec<String>>,
    /// Instructions for the user
    pub instructions: String,
}

/// Logical challenge data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalChallenge {
    /// The logical problem to solve
    pub problem: String,
    /// Expected answer to the problem
    pub answer: String,
    /// Type of logical challenge
    pub challenge_type: String,
    /// Instructions for solving the problem
    pub instructions: String,
}

/// Mathematical operation types for logical challenges
#[derive(Debug, Clone)]
enum MathOperation {
    Addition,
    Subtraction,
    Multiplication,
    Sequence,
    Pattern,
}

/// Challenge generator trait
#[async_trait]
pub trait ChallengeGeneratorTrait: Send + Sync {
    /// Generate a visual challenge
    async fn generate_visual_challenge(&self, difficulty: u8) -> Result<String, CaptchaError>;

    /// Generate an audio challenge
    async fn generate_audio_challenge(&self, difficulty: u8) -> Result<String, CaptchaError>;

    /// Generate a logical challenge
    async fn generate_logical_challenge(&self, difficulty: u8) -> Result<String, CaptchaError>;

    /// Generate a behavioral challenge
    async fn generate_behavioral_challenge(&self, difficulty: u8) -> Result<String, CaptchaError>;

    /// Generate a hybrid challenge
    async fn generate_hybrid_challenge(&self, difficulty: u8) -> Result<String, CaptchaError>;

    /// Generate a complete challenge with metadata
    async fn generate_challenge(
        &self,
        challenge_type: ChallengeType,
        difficulty: u8,
        session_id: Option<String>,
    ) -> Result<Challenge, CaptchaError>;

    /// Generate a challenge with adaptive difficulty
    async fn generate_adaptive_challenge(
        &self,
        challenge_type: ChallengeType,
        session_id: Option<String>,
        ip_address: &str,
        behavioral_metrics: Option<&BehavioralMetrics>,
        difficulty_calculator: &mut AdaptiveDifficultyCalculator,
    ) -> Result<Challenge, CaptchaError>;
}

/// Default challenge generator implementation with cryptographic security
pub struct ChallengeGenerator {
    /// Cryptographically secure random number generator
    rng: ChaCha20Rng,
    /// Image challenge generator
    image_generator: ImageChallengeGenerator,
    /// Audio challenge generator
    audio_generator: AudioChallengeGenerator,
}

impl ChallengeGenerator {
    /// Create a new challenge generator with cryptographically secure RNG
    pub fn new() -> Self {
        // Use system entropy to seed the cryptographically secure RNG
        let rng = ChaCha20Rng::from_entropy();
        Self {
            rng,
            image_generator: ImageChallengeGenerator::new(),
            audio_generator: AudioChallengeGenerator::new(),
        }
    }

    /// Generate an advanced image challenge
    pub fn generate_advanced_image_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<String, CaptchaError> {
        let challenge = self.image_generator.generate_random_challenge(difficulty)?;
        serde_json::to_string(&challenge).map_err(|e| CaptchaError::GenerationFailed {
            message: format!("Serialization failed: {}", e),
            recoverable: true,
            retry_after: Some(Duration::from_secs(1)),
        })
    }

    /// Generate an advanced audio challenge
    pub fn generate_advanced_audio_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<String, CaptchaError> {
        let challenge = self.audio_generator.generate_random_challenge(difficulty)?;
        serde_json::to_string(&challenge).map_err(|e| CaptchaError::GenerationFailed {
            message: format!("Serialization failed: {}", e),
            recoverable: true,
            retry_after: Some(Duration::from_secs(1)),
        })
    }

    /// Generate a secure hash for the expected answer
    fn hash_answer(&self, answer: &str, challenge_id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(answer.as_bytes());
        hasher.update(challenge_id.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Generate visual text-based challenge
    fn generate_text_challenge(&mut self, difficulty: u8) -> Result<VisualChallenge, CaptchaError> {
        let length = match difficulty {
            1..=3 => 4,
            4..=6 => 5,
            7..=8 => 6,
            9..=10 => 7,
            _ => {
                return Err(CaptchaError::GenerationFailed {
                    message: "Invalid difficulty level".to_string(),
                    recoverable: true,
                    retry_after: Some(Duration::from_secs(1)),
                });
            }
        };

        // Generate random alphanumeric string
        let chars: Vec<char> = "ABCDEFGHIJKLMNPQRSTUVWXYZ23456789".chars().collect();
        let mut answer = String::new();

        for _ in 0..length {
            let idx = self.rng.gen_range(0..chars.len());
            answer.push(chars[idx]);
        }

        Ok(VisualChallenge {
            challenge_type: "text_recognition".to_string(),
            data: answer.clone(),
            answer: answer.clone(),
            options: None,
            instructions: "Enter the characters you see".to_string(),
        })
    }

    /// Generate visual image selection challenge
    fn generate_image_selection_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<VisualChallenge, CaptchaError> {
        let categories = vec![
            "traffic_lights",
            "crosswalks",
            "vehicles",
            "bicycles",
            "fire_hydrants",
            "stairs",
            "bridges",
            "mountains",
        ];

        let category_idx = self.rng.gen_range(0..categories.len());
        let selected_category = categories[category_idx];

        // Generate grid size based on difficulty
        let grid_size = match difficulty {
            1..=3 => 3,
            4..=6 => 4,
            7..=10 => 5,
            _ => {
                return Err(CaptchaError::GenerationFailed {
                    message: "Invalid difficulty level".to_string(),
                    recoverable: true,
                    retry_after: Some(Duration::from_secs(1)),
                });
            }
        };

        // Generate correct answer positions
        let total_images = grid_size * grid_size;
        let num_correct = self.rng.gen_range(1..=(total_images / 3).max(1));
        let mut correct_positions = Vec::new();

        for _ in 0..num_correct {
            let pos = self.rng.gen_range(0..total_images);
            if !correct_positions.contains(&pos) {
                correct_positions.push(pos);
            }
        }

        correct_positions.sort();
        let answer = correct_positions
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        Ok(VisualChallenge {
            challenge_type: "image_selection".to_string(),
            data: serde_json::json!({
                "category": selected_category,
                "grid_size": grid_size,
                "total_images": total_images
            })
            .to_string(),
            answer,
            options: None,
            instructions: format!(
                "Select all images with {}",
                selected_category.replace("_", " ")
            ),
        })
    }

    /// Generate mathematical challenge
    fn generate_math_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<LogicalChallenge, CaptchaError> {
        let operation = match difficulty {
            1..=3 => MathOperation::Addition,
            4..=5 => MathOperation::Subtraction,
            6..=7 => MathOperation::Multiplication,
            8..=9 => MathOperation::Sequence,
            10 => MathOperation::Pattern,
            _ => {
                return Err(CaptchaError::GenerationFailed {
                    message: "Invalid difficulty level".to_string(),
                    recoverable: true,
                    retry_after: Some(Duration::from_secs(1)),
                });
            }
        };

        match operation {
            MathOperation::Addition => {
                let a = self.rng.gen_range(1..=50);
                let b = self.rng.gen_range(1..=50);
                let answer = a + b;
                Ok(LogicalChallenge {
                    problem: format!("{} + {} = ?", a, b),
                    answer: answer.to_string(),
                    challenge_type: "math_addition".to_string(),
                    instructions: "Solve the mathematical equation".to_string(),
                })
            }
            MathOperation::Subtraction => {
                let a = self.rng.gen_range(20..=100);
                let b = self.rng.gen_range(1..=a);
                let answer = a - b;
                Ok(LogicalChallenge {
                    problem: format!("{} - {} = ?", a, b),
                    answer: answer.to_string(),
                    challenge_type: "math_subtraction".to_string(),
                    instructions: "Solve the mathematical equation".to_string(),
                })
            }
            MathOperation::Multiplication => {
                let a = self.rng.gen_range(2..=12);
                let b = self.rng.gen_range(2..=12);
                let answer = a * b;
                Ok(LogicalChallenge {
                    problem: format!("{} × {} = ?", a, b),
                    answer: answer.to_string(),
                    challenge_type: "math_multiplication".to_string(),
                    instructions: "Solve the mathematical equation".to_string(),
                })
            }
            MathOperation::Sequence => {
                let start = self.rng.gen_range(1..=10);
                let step = self.rng.gen_range(2..=5);
                let sequence: Vec<i32> = (0..4).map(|i| start + i * step).collect();
                let next = start + 4 * step;

                Ok(LogicalChallenge {
                    problem: format!(
                        "{}, {}, {}, {}, ?",
                        sequence[0], sequence[1], sequence[2], sequence[3]
                    ),
                    answer: next.to_string(),
                    challenge_type: "number_sequence".to_string(),
                    instructions: "Complete the number sequence".to_string(),
                })
            }
            MathOperation::Pattern => {
                // Fibonacci-like pattern
                let a = self.rng.gen_range(1..=3);
                let b = self.rng.gen_range(1..=3);
                let c = a + b;
                let d = b + c;
                let e = c + d;

                Ok(LogicalChallenge {
                    problem: format!("{}, {}, {}, {}, ?", a, b, c, d),
                    answer: e.to_string(),
                    challenge_type: "pattern_recognition".to_string(),
                    instructions: "Find the pattern and complete the sequence".to_string(),
                })
            }
        }
    }

    /// Generate word-based logical challenge
    fn generate_word_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<LogicalChallenge, CaptchaError> {
        let word_pairs = vec![
            ("cat", "animal"),
            ("rose", "flower"),
            ("car", "vehicle"),
            ("apple", "fruit"),
            ("chair", "furniture"),
            ("book", "object"),
            ("dog", "animal"),
            ("tree", "plant"),
            ("house", "building"),
        ];

        let pair_idx = self.rng.gen_range(0..word_pairs.len());
        let (word, category) = word_pairs[pair_idx];

        // Create multiple choice options
        let mut options = vec![category.to_string()];
        let distractors = vec!["color", "number", "emotion", "weather", "food", "tool"];

        for distractor in distractors.iter().take(3) {
            if *distractor != category {
                options.push(distractor.to_string());
            }
        }

        // Shuffle options
        for i in (1..options.len()).rev() {
            let j = self.rng.gen_range(0..=i);
            options.swap(i, j);
        }

        Ok(LogicalChallenge {
            problem: format!(
                "What category does '{}' belong to? Options: {}",
                word,
                options.join(", ")
            ),
            answer: category.to_string(),
            challenge_type: "word_categorization".to_string(),
            instructions: "Select the correct category for the given word".to_string(),
        })
    }
}

impl Default for ChallengeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChallengeGeneratorTrait for ChallengeGenerator {
    async fn generate_visual_challenge(&self, difficulty: u8) -> Result<String, CaptchaError> {
        let mut generator = Self::new(); // Create new instance for thread safety

        // Choose between basic and advanced image challenges based on difficulty
        let challenge_variant = if difficulty >= 5 {
            // Higher difficulty - use advanced challenges more often
            generator.rng.gen_range(0..4)
        } else {
            // Lower difficulty - use basic challenges
            generator.rng.gen_range(0..2)
        };

        match challenge_variant {
            0 => {
                let visual_challenge = generator.generate_text_challenge(difficulty)?;
                serde_json::to_string(&visual_challenge).map_err(|e| {
                    CaptchaError::GenerationFailed {
                        message: format!("Serialization failed: {}", e),
                        recoverable: true,
                        retry_after: Some(Duration::from_secs(1)),
                    }
                })
            }
            1 => {
                let visual_challenge = generator.generate_image_selection_challenge(difficulty)?;
                serde_json::to_string(&visual_challenge).map_err(|e| {
                    CaptchaError::GenerationFailed {
                        message: format!("Serialization failed: {}", e),
                        recoverable: true,
                        retry_after: Some(Duration::from_secs(1)),
                    }
                })
            }
            _ => {
                // Use advanced image challenges
                generator.generate_advanced_image_challenge(difficulty)
            }
        }
    }

    async fn generate_audio_challenge(&self, difficulty: u8) -> Result<String, CaptchaError> {
        let mut generator = Self::new(); // Create new instance for thread safety

        // Use advanced audio challenges
        generator.generate_advanced_audio_challenge(difficulty)
    }

    async fn generate_logical_challenge(&self, difficulty: u8) -> Result<String, CaptchaError> {
        let mut generator = Self::new(); // Create new instance for thread safety

        // Choose between math and word challenges based on difficulty
        let logical_challenge = if difficulty <= 7 {
            generator.generate_math_challenge(difficulty)?
        } else {
            // Mix of math and word challenges for higher difficulty
            let challenge_type = generator.rng.gen_range(0..2);
            match challenge_type {
                0 => generator.generate_math_challenge(difficulty)?,
                1 => generator.generate_word_challenge(difficulty)?,
                _ => unreachable!(),
            }
        };

        serde_json::to_string(&logical_challenge).map_err(|e| CaptchaError::GenerationFailed {
            message: format!("Serialization failed: {}", e),
            recoverable: true,
            retry_after: Some(Duration::from_secs(1)),
        })
    }

    async fn generate_behavioral_challenge(&self, difficulty: u8) -> Result<String, CaptchaError> {
        // Behavioral challenge placeholder - requires frontend interaction tracking
        let behavioral_challenge = serde_json::json!({
            "challenge_type": "mouse_movement",
            "instructions": "Move your mouse in a natural pattern to complete this challenge",
            "required_actions": ["mouse_move", "click_sequence"],
            "difficulty": difficulty,
            "timeout": 30000 // 30 seconds
        });

        Ok(behavioral_challenge.to_string())
    }

    async fn generate_hybrid_challenge(&self, difficulty: u8) -> Result<String, CaptchaError> {
        let mut generator = Self::new();

        // Combine visual and logical challenges
        let visual = generator.generate_text_challenge(difficulty.min(5))?;
        let logical = generator.generate_math_challenge(difficulty.max(3))?;

        let hybrid_challenge = serde_json::json!({
            "challenge_type": "hybrid",
            "visual_component": visual,
            "logical_component": logical,
            "instructions": "Complete both the visual and logical challenges",
            "difficulty": difficulty
        });

        Ok(hybrid_challenge.to_string())
    }

    async fn generate_challenge(
        &self,
        challenge_type: ChallengeType,
        difficulty: u8,
        session_id: Option<String>,
    ) -> Result<Challenge, CaptchaError> {
        // Validate difficulty level
        if !(1..=10).contains(&difficulty) {
            return Err(CaptchaError::GenerationFailed {
                message: "Difficulty must be between 1 and 10".to_string(),
                recoverable: true,
                retry_after: Some(Duration::from_secs(1)),
            });
        }

        // Generate challenge data based on type
        let challenge_data = match challenge_type {
            ChallengeType::Visual => self.generate_visual_challenge(difficulty).await?,
            ChallengeType::Audio => self.generate_audio_challenge(difficulty).await?,
            ChallengeType::Logical => self.generate_logical_challenge(difficulty).await?,
            ChallengeType::Behavioral => self.generate_behavioral_challenge(difficulty).await?,
            ChallengeType::Hybrid => self.generate_hybrid_challenge(difficulty).await?,
        };

        // Extract expected answer from challenge data for hashing
        let expected_answer =
            self.extract_answer_from_challenge(&challenge_data, &challenge_type)?;

        // Create challenge ID for hashing
        let challenge_id = uuid::Uuid::new_v4().to_string();

        // Hash the expected answer
        let expected_answer_hash = self.hash_answer(&expected_answer, &challenge_id);

        // Create challenge with unencrypted data for now
        // Note: Secreton integration is available via CaptchaSecretonClient
        // In production, challenge_data should be encrypted before storage
        let challenge = Challenge::new(
            challenge_type,
            difficulty,
            challenge_data, // TODO: Encrypt using CaptchaSecretonClient in production
            expected_answer_hash,
            session_id,
            "127.0.0.1".to_string(), // Placeholder IP - will be extracted from request context
        );

        Ok(challenge)
    }

    async fn generate_adaptive_challenge(
        &self,
        challenge_type: ChallengeType,
        session_id: Option<String>,
        ip_address: &str,
        behavioral_metrics: Option<&BehavioralMetrics>,
        difficulty_calculator: &mut AdaptiveDifficultyCalculator,
    ) -> Result<Challenge, CaptchaError> {
        // Get session ID or create a temporary one
        let session_id_str = session_id.as_deref().unwrap_or("anonymous");

        // Perform threat assessment
        let threat_assessment = difficulty_calculator.assess_threat(session_id_str, ip_address);

        // Calculate adaptive difficulty
        let adaptive_difficulty = difficulty_calculator.calculate_difficulty(
            session_id_str,
            ip_address,
            behavioral_metrics,
            Some(&threat_assessment),
        );

        // Generate challenge with calculated difficulty
        self.generate_challenge(challenge_type, adaptive_difficulty, session_id)
            .await
    }
}

impl ChallengeGenerator {
    /// Extract the expected answer from challenge data for hashing
    fn extract_answer_from_challenge(
        &self,
        challenge_data: &str,
        challenge_type: &ChallengeType,
    ) -> Result<String, CaptchaError> {
        match challenge_type {
            ChallengeType::Visual => {
                let visual: VisualChallenge =
                    serde_json::from_str(challenge_data).map_err(|e| {
                        CaptchaError::GenerationFailed {
                            message: format!("Failed to parse visual challenge: {}", e),
                            recoverable: true,
                            retry_after: Some(Duration::from_secs(1)),
                        }
                    })?;
                Ok(visual.answer)
            }
            ChallengeType::Logical => {
                let logical: LogicalChallenge =
                    serde_json::from_str(challenge_data).map_err(|e| {
                        CaptchaError::GenerationFailed {
                            message: format!("Failed to parse logical challenge: {}", e),
                            recoverable: true,
                            retry_after: Some(Duration::from_secs(1)),
                        }
                    })?;
                Ok(logical.answer)
            }
            ChallengeType::Audio | ChallengeType::Behavioral | ChallengeType::Hybrid => {
                // For now, return a placeholder - these will be implemented in future tasks
                Ok("placeholder_answer".to_string())
            }
        }
    }
}
