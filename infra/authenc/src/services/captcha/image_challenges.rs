//! Advanced Image-Basedypes
//!
//! Implements sophisticated image-based puzzles including jigsaw, rotation, and object selection

use super::error::CaptchaError;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Image-based puzzle challenge types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ImagePuzzleType {
    Jigsaw {
        /// Number of pieces in the puzzle
        pieces: u8,
        /// Grid dimensions (e.g., 3x3, 4x4)
        grid_size: u8,
        /// Piece positions that need to be arranged
        scrambled_positions: Vec<u8>,
        /// Correct order of pieces
        correct_order: Vec<u8>,
    },
    /// Rotation puzzle - rotate image to correct orientation
    Rotation {
        /// Current rotation angle in degrees (0, 90, 180, 270)
        current_angle: u16,
        /// Correct rotation angle
        correct_angle: u16,
        /// Image identifier
        image_id: String,
    },
    /// Object selection - select specific objects in image
    ObjectSelection {
        /// Category of objects to select
        target_category: String,
        /// Total number of selectable regions
        total_regions: u8,
        /// Indices of regions containing target objects
        correct_regions: Vec<u8>,
        /// Grid layout (rows x cols)
        grid_layout: (u8, u8),
    },
    /// Sequence ordering - arrange images in correct sequence
    SequenceOrdering {
        /// Number of images in sequence
        sequence_length: u8,
        /// Scrambled order of images
        scrambled_order: Vec<u8>,
        /// Correct sequence order
        correct_sequence: Vec<u8>,
        /// Sequence type (e.g., "time_progression", "size_order")
        sequence_type: String,
    },
}

/// Image challenge data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Jigsaw puzzle - arrange pieces to form complete image
pub struct ImageChallenge {
    /// Type of image puzzle
    pub puzzle_type: ImagePuzzleType,
    /// Instructions for the user
    pub instructions: String,
    /// Base image URL or identifier
    pub image_url: String,
    /// Expected answer (serialized solution)
    pub answer: String,
    /// Difficulty level (1-10)
    pub difficulty: u8,
    /// Time limit in seconds
    pub time_limit: u32,
}

/// Image challenge generator
pub struct ImageChallengeGenerator {
    rng: ChaCha20Rng,
}

impl ImageChallengeGenerator {
    /// Create a new image challenge generator
    pub fn new() -> Self {
        Self {
            rng: ChaCha20Rng::from_entropy(),
        }
    }

    /// Generate a jigsaw puzzle challenge
    pub fn generate_jigsaw_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<ImageChallenge, CaptchaError> {
        // Determine grid size based on difficulty
        let grid_size = match difficulty {
            1..=3 => 2,  // 2x2 = 4 pieces
            4..=6 => 3,  // 3x3 = 9 pieces
            7..=8 => 4,  // 4x4 = 16 pieces
            9..=10 => 5, // 5x5 = 25 pieces
            _ => {
                return Err(CaptchaError::GenerationFailed {
                    message: "Invalid difficulty level".to_string(),
                    recoverable: true,
                    retry_after: Some(Duration::from_secs(1)),
                });
            }
        };

        let pieces = grid_size * grid_size;

        // Generate correct order
        let correct_order: Vec<u8> = (0..pieces).collect();

        // Scramble positions
        let mut scrambled_positions = correct_order.clone();
        for i in (1..scrambled_positions.len()).rev() {
            let j = self.rng.gen_range(0..=i);
            scrambled_positions.swap(i, j);
        }

        // Ensure it's actually scrambled (not already solved)
        if scrambled_positions == correct_order {
            scrambled_positions.swap(0, 1);
        }

        let puzzle_type = ImagePuzzleType::Jigsaw {
            pieces,
            grid_size,
            scrambled_positions: scrambled_positions.clone(),
            correct_order: correct_order.clone(),
        };

        // Answer is the correct order as comma-separated string
        let answer = correct_order
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let time_limit = match difficulty {
            1..=3 => 60,
            4..=6 => 90,
            7..=8 => 120,
            _ => 180,
        };

        Ok(ImageChallenge {
            puzzle_type,
            instructions: format!(
                "Arrange the {} puzzle pieces to form the complete image",
                pieces
            ),
            image_url: format!("/api/v1/captcha/images/jigsaw/{}", uuid::Uuid::new_v4()),
            answer,
            difficulty,
            time_limit,
        })
    }

    /// Generate a rotation challenge
    pub fn generate_rotation_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<ImageChallenge, CaptchaError> {
        // Possible rotation angles
        let angles = [0, 90, 180, 270];

        // Correct angle is always 0 (upright)
        let correct_angle = 0;

        // Current angle is randomly rotated
        let current_angle = angles[self.rng.gen_range(1..angles.len())];

        let puzzle_type = ImagePuzzleType::Rotation {
            current_angle,
            correct_angle,
            image_id: uuid::Uuid::new_v4().to_string(),
        };

        let time_limit = match difficulty {
            1..=5 => 30,
            6..=8 => 45,
            _ => 60,
        };

        Ok(ImageChallenge {
            puzzle_type,
            instructions: "Rotate the image to its correct upright orientation".to_string(),
            image_url: format!("/api/v1/captcha/images/rotation/{}", uuid::Uuid::new_v4()),
            answer: correct_angle.to_string(),
            difficulty,
            time_limit,
        })
    }

    /// Generate an object selection challenge
    pub fn generate_object_selection_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<ImageChallenge, CaptchaError> {
        // Object categories
        let categories = vec![
            "traffic_lights",
            "crosswalks",
            "vehicles",
            "bicycles",
            "fire_hydrants",
            "stairs",
            "bridges",
            "mountains",
            "trees",
            "buildings",
            "animals",
            "people",
        ];

        let category_idx = self.rng.gen_range(0..categories.len());
        let target_category = categories[category_idx].to_string();

        // Determine grid layout based on difficulty
        let (rows, cols) = match difficulty {
            1..=3 => (3, 3),  // 9 regions
            4..=6 => (4, 4),  // 16 regions
            7..=8 => (4, 5),  // 20 regions
            9..=10 => (5, 5), // 25 regions
            _ => {
                return Err(CaptchaError::GenerationFailed {
                    message: "Invalid difficulty level".to_string(),
                    recoverable: true,
                    retry_after: Some(Duration::from_secs(1)),
                });
            }
        };

        let total_regions = rows * cols;

        // Determine number of correct regions (20-40% of total)
        let min_correct = (total_regions as f32 * 0.2) as u8;
        let max_correct = (total_regions as f32 * 0.4) as u8;
        let num_correct = self.rng.gen_range(min_correct.max(1)..=max_correct.max(2));

        // Generate correct region indices
        let mut correct_regions = Vec::new();
        while correct_regions.len() < num_correct as usize {
            let region = self.rng.gen_range(0..total_regions);
            if !correct_regions.contains(&region) {
                correct_regions.push(region);
            }
        }
        correct_regions.sort();

        let puzzle_type = ImagePuzzleType::ObjectSelection {
            target_category: target_category.clone(),
            total_regions,
            correct_regions: correct_regions.clone(),
            grid_layout: (rows, cols),
        };

        let answer = correct_regions
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let time_limit = match difficulty {
            1..=3 => 45,
            4..=6 => 60,
            7..=8 => 75,
            _ => 90,
        };

        Ok(ImageChallenge {
            puzzle_type,
            instructions: format!(
                "Select all images containing {}",
                target_category.replace("_", " ")
            ),
            image_url: format!("/api/v1/captcha/images/selection/{}", uuid::Uuid::new_v4()),
            answer,
            difficulty,
            time_limit,
        })
    }

    /// Generate a sequence ordering challenge
    pub fn generate_sequence_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<ImageChallenge, CaptchaError> {
        // Sequence types
        let sequence_types = vec![
            "time_progression",
            "size_order",
            "age_progression",
            "growth_stages",
            "assembly_steps",
        ];

        let sequence_type = sequence_types[self.rng.gen_range(0..sequence_types.len())].to_string();

        // Determine sequence length based on difficulty
        let sequence_length = match difficulty {
            1..=3 => 3,
            4..=6 => 4,
            7..=8 => 5,
            9..=10 => 6,
            _ => {
                return Err(CaptchaError::GenerationFailed {
                    message: "Invalid difficulty level".to_string(),
                    recoverable: true,
                    retry_after: Some(Duration::from_secs(1)),
                });
            }
        };

        // Generate correct sequence
        let correct_sequence: Vec<u8> = (0..sequence_length).collect();

        // Scramble the sequence
        let mut scrambled_order = correct_sequence.clone();
        for i in (1..scrambled_order.len()).rev() {
            let j = self.rng.gen_range(0..=i);
            scrambled_order.swap(i, j);
        }

        // Ensure it's scrambled
        if scrambled_order == correct_sequence {
            scrambled_order.swap(0, 1);
        }

        let puzzle_type = ImagePuzzleType::SequenceOrdering {
            sequence_length,
            scrambled_order: scrambled_order.clone(),
            correct_sequence: correct_sequence.clone(),
            sequence_type: sequence_type.clone(),
        };

        let answer = correct_sequence
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let time_limit = match difficulty {
            1..=3 => 60,
            4..=6 => 75,
            7..=8 => 90,
            _ => 120,
        };

        Ok(ImageChallenge {
            puzzle_type,
            instructions: format!(
                "Arrange the images in the correct {} order",
                sequence_type.replace("_", " ")
            ),
            image_url: format!("/api/v1/captcha/images/sequence/{}", uuid::Uuid::new_v4()),
            answer,
            difficulty,
            time_limit,
        })
    }

    /// Generate a random image challenge based on difficulty
    pub fn generate_random_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<ImageChallenge, CaptchaError> {
        // Weight different challenge types based on difficulty
        let challenge_type = if difficulty <= 3 {
            // Easier challenges for low difficulty
            self.rng.gen_range(0..2) // Rotation or simple object selection
        } else if difficulty <= 6 {
            self.rng.gen_range(0..3) // Add jigsaw
        } else {
            self.rng.gen_range(0..4) // All types including sequence
        };

        match challenge_type {
            0 => self.generate_rotation_challenge(difficulty),
            1 => self.generate_object_selection_challenge(difficulty),
            2 => self.generate_jigsaw_challenge(difficulty),
            3 => self.generate_sequence_challenge(difficulty),
            _ => unreachable!(),
        }
    }
}

impl Default for ImageChallengeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jigsaw_challenge_generation() {
        let mut generator = ImageChallengeGenerator::new();

        for difficulty in 1..=10 {
            let challenge = generator.generate_jigsaw_challenge(difficulty);
            assert!(challenge.is_ok());

            let challenge = challenge.unwrap();
            assert_eq!(challenge.difficulty, difficulty);
            assert!(!challenge.answer.is_empty());
        }
    }

    #[test]
    fn test_rotation_challenge_generation() {
        let mut generator = ImageChallengeGenerator::new();

        let challenge = generator.generate_rotation_challenge(5).unwrap();
        assert_eq!(challenge.difficulty, 5);
        assert_eq!(challenge.answer, "0"); // Correct angle is always 0
    }

    #[test]
    fn test_object_selection_challenge_generation() {
        let mut generator = ImageChallengeGenerator::new();

        for difficulty in 1..=10 {
            let challenge = generator.generate_object_selection_challenge(difficulty);
            assert!(challenge.is_ok());

            let challenge = challenge.unwrap();
            assert_eq!(challenge.difficulty, difficulty);

            // Verify answer format (comma-separated numbers)
            let regions: Vec<&str> = challenge.answer.split(',').collect();
            assert!(!regions.is_empty());
        }
    }

    #[test]
    fn test_sequence_challenge_generation() {
        let mut generator = ImageChallengeGenerator::new();

        for difficulty in 1..=10 {
            let challenge = generator.generate_sequence_challenge(difficulty);
            assert!(challenge.is_ok());

            let challenge = challenge.unwrap();
            assert_eq!(challenge.difficulty, difficulty);

            // Verify answer is a valid sequence
            let sequence: Vec<&str> = challenge.answer.split(',').collect();
            assert!(sequence.len() >= 3);
        }
    }

    #[test]
    fn test_random_challenge_generation() {
        let mut generator = ImageChallengeGenerator::new();

        // Generate multiple challenges to test randomness
        for _ in 0..10 {
            let challenge = generator.generate_random_challenge(5);
            assert!(challenge.is_ok());
        }
    }
}
