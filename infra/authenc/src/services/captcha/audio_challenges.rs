//! Advanced Audio-Based Challenge Types
//!
//! Implements audio pattern recognition challenges including tone sequences and spoken words

use super::error::CaptchaError;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Audio challenge types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AudioChallengeType {
    ToneSequence {
        /// Sequence of tone frequencies in Hz
        tone_frequencies: Vec<u16>,
        /// Duration of each tone in milliseconds
        tone_duration_ms: u16,
        /// Pause between tones in milliseconds
        pause_duration_ms: u16,
    },
    /// Spoken digits - recognize spoken number sequence
    SpokenDigits {
        /// Sequence of digits to be spoken
        digit_sequence: Vec<u8>,
        /// Voice type (male, female, child)
        voice_type: String,
        /// Speech rate (slow, normal, fast)
        speech_rate: String,
    },
    /// Spoken words - recognize spoken words
    SpokenWords {
        /// Words to be spoken
        words: Vec<String>,
        /// Voice type
        voice_type: String,
        /// Background noise level (none, low, medium, high)
        noise_level: String,
    },
    /// Pattern recognition - identify repeating audio pattern
    PatternRecognition {
        /// Pattern elements (tone frequencies or word indices)
        pattern: Vec<u16>,
        /// Number of times pattern repeats
        repetitions: u8,
        /// Type of pattern (ascending, descending, alternating)
        pattern_type: String,
    },
    /// Sound identification - identify specific sounds
    SoundIdentification {
        /// Category of sounds to identify
        sound_category: String,
        /// Number of sounds in sequence
        sound_count: u8,
        /// Sounds to identify (indices or identifiers)
        target_sounds: Vec<String>,
    },
}

/// Audio challenge data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Tone sequence - identify sequence of musical tones
pub struct AudioChallenge {
    /// Type of audio challenge
    pub challenge_type: AudioChallengeType,
    /// Instructions for the user
    pub instructions: String,
    /// Audio file URL or identifier
    pub audio_url: String,
    /// Expected answer
    pub answer: String,
    /// Difficulty level (1-10)
    pub difficulty: u8,
    /// Time limit in seconds
    pub time_limit: u32,
    /// Whether audio can be replayed
    pub allow_replay: bool,
    /// Maximum number of replays allowed
    pub max_replays: u8,
}

/// Audio challenge generator
pub struct AudioChallengeGenerator {
    rng: ChaCha20Rng,
}

impl AudioChallengeGenerator {
    /// Create a new audio challenge generator
    pub fn new() -> Self {
        Self {
            rng: ChaCha20Rng::from_entropy(),
        }
    }

    /// Generate a tone sequence challenge
    pub fn generate_tone_sequence_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<AudioChallenge, CaptchaError> {
        // Musical note frequencies (C4 to C5)
        let base_frequencies = [262, 294, 330, 349, 392, 440, 494, 523];

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

        // Generate random tone sequence
        let mut tone_frequencies = Vec::new();
        for _ in 0..sequence_length {
            let freq_idx = self.rng.gen_range(0..base_frequencies.len());
            tone_frequencies.push(base_frequencies[freq_idx]);
        }

        // Tone duration decreases with difficulty
        let tone_duration_ms = match difficulty {
            1..=3 => 800,
            4..=6 => 600,
            7..=8 => 400,
            _ => 300,
        };

        let pause_duration_ms = tone_duration_ms / 2;

        let challenge_type = AudioChallengeType::ToneSequence {
            tone_frequencies: tone_frequencies.clone(),
            tone_duration_ms,
            pause_duration_ms,
        };

        // Answer is the sequence of tone indices
        let answer = tone_frequencies
            .iter()
            .map(|freq| {
                base_frequencies
                    .iter()
                    .position(|&f| f == *freq)
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(",");

        let time_limit = 60 + (sequence_length as u32 * 10);
        let max_replays = match difficulty {
            1..=5 => 3,
            6..=8 => 2,
            _ => 1,
        };

        Ok(AudioChallenge {
            challenge_type,
            instructions: "Listen to the tone sequence and identify the notes".to_string(),
            audio_url: format!("/api/v1/captcha/audio/tones/{}", uuid::Uuid::new_v4()),
            answer,
            difficulty,
            time_limit,
            allow_replay: true,
            max_replays,
        })
    }

    /// Generate a spoken digits challenge
    pub fn generate_spoken_digits_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<AudioChallenge, CaptchaError> {
        // Determine digit sequence length based on difficulty
        let sequence_length = match difficulty {
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

        // Generate random digit sequence
        let mut digit_sequence = Vec::new();
        for _ in 0..sequence_length {
            digit_sequence.push(self.rng.gen_range(0..10));
        }

        // Voice type selection
        let voice_types = ["male", "female", "child"];
        let voice_type = voice_types[self.rng.gen_range(0..voice_types.len())].to_string();

        // Speech rate based on difficulty
        let speech_rate = match difficulty {
            1..=4 => "slow",
            5..=7 => "normal",
            _ => "fast",
        }
        .to_string();

        let challenge_type = AudioChallengeType::SpokenDigits {
            digit_sequence: digit_sequence.clone(),
            voice_type,
            speech_rate,
        };

        let answer = digit_sequence
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join("");

        let time_limit = 45 + (sequence_length as u32 * 5);
        let max_replays = match difficulty {
            1..=5 => 2,
            _ => 1,
        };

        Ok(AudioChallenge {
            challenge_type,
            instructions: "Listen carefully and enter the digits you hear".to_string(),
            audio_url: format!("/api/v1/captcha/audio/digits/{}", uuid::Uuid::new_v4()),
            answer,
            difficulty,
            time_limit,
            allow_replay: true,
            max_replays,
        })
    }

    /// Generate a spoken words challenge
    pub fn generate_spoken_words_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<AudioChallenge, CaptchaError> {
        // Word lists by difficulty
        let easy_words = vec!["cat", "dog", "sun", "moon", "tree", "book", "car", "house"];
        let medium_words = vec![
            "elephant", "computer", "mountain", "ocean", "library", "garden",
        ];
        let hard_words = vec![
            "architecture",
            "philosophy",
            "technology",
            "environment",
            "democracy",
        ];

        let word_list = match difficulty {
            1..=3 => &easy_words,
            4..=7 => &medium_words,
            _ => &hard_words,
        };

        // Number of words based on difficulty
        let word_count = match difficulty {
            1..=3 => 2,
            4..=6 => 3,
            7..=8 => 4,
            _ => 5,
        };

        // Select random words
        let mut words = Vec::new();
        for _ in 0..word_count {
            let word_idx = self.rng.gen_range(0..word_list.len());
            words.push(word_list[word_idx].to_string());
        }

        let voice_types = ["male", "female"];
        let voice_type = voice_types[self.rng.gen_range(0..voice_types.len())].to_string();

        // Noise level increases with difficulty
        let noise_level = match difficulty {
            1..=4 => "none",
            5..=7 => "low",
            8..=9 => "medium",
            _ => "high",
        }
        .to_string();

        let challenge_type = AudioChallengeType::SpokenWords {
            words: words.clone(),
            voice_type,
            noise_level,
        };

        let answer = words.join(" ");

        let time_limit = 60 + (word_count as u32 * 10);
        let max_replays = match difficulty {
            1..=6 => 2,
            _ => 1,
        };

        Ok(AudioChallenge {
            challenge_type,
            instructions: "Listen to the spoken words and type them in order".to_string(),
            audio_url: format!("/api/v1/captcha/audio/words/{}", uuid::Uuid::new_v4()),
            answer,
            difficulty,
            time_limit,
            allow_replay: true,
            max_replays,
        })
    }

    /// Generate a pattern recognition challenge
    pub fn generate_pattern_recognition_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<AudioChallenge, CaptchaError> {
        // Pattern types
        let pattern_types = ["ascending", "descending", "alternating", "repeating"];
        let pattern_type = pattern_types[self.rng.gen_range(0..pattern_types.len())].to_string();

        // Pattern length based on difficulty
        let pattern_length = match difficulty {
            1..=3 => 3,
            4..=6 => 4,
            7..=8 => 5,
            _ => 6,
        };

        // Generate pattern based on type
        let pattern = match pattern_type.as_str() {
            "ascending" => {
                let start = self.rng.gen_range(200..400);
                (0..pattern_length)
                    .map(|i| start + (i as u16 * 50))
                    .collect()
            }
            "descending" => {
                let start = self.rng.gen_range(500..700);
                (0..pattern_length)
                    .map(|i| start - (i as u16 * 50))
                    .collect()
            }
            "alternating" => {
                let low = self.rng.gen_range(200..300);
                let high = self.rng.gen_range(500..600);
                (0..pattern_length)
                    .map(|i| if i % 2 == 0 { low } else { high })
                    .collect()
            }
            _ => {
                // repeating
                let tone = self.rng.gen_range(300..500);
                vec![tone; pattern_length as usize]
            }
        };

        let repetitions = match difficulty {
            1..=5 => 2,
            6..=8 => 3,
            _ => 4,
        };

        let challenge_type = AudioChallengeType::PatternRecognition {
            pattern: pattern.clone(),
            repetitions,
            pattern_type: pattern_type.clone(),
        };

        let answer = pattern_type;

        let time_limit = 60 + (pattern_length as u32 * repetitions as u32 * 5);

        Ok(AudioChallenge {
            challenge_type,
            instructions: "Listen to the audio pattern and identify its type".to_string(),
            audio_url: format!("/api/v1/captcha/audio/pattern/{}", uuid::Uuid::new_v4()),
            answer,
            difficulty,
            time_limit,
            allow_replay: true,
            max_replays: 2,
        })
    }

    /// Generate a sound identification challenge
    pub fn generate_sound_identification_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<AudioChallenge, CaptchaError> {
        // Sound categories
        let categories = vec![
            (
                "animals",
                vec!["dog_bark", "cat_meow", "bird_chirp", "cow_moo"],
            ),
            ("instruments", vec!["piano", "guitar", "drums", "violin"]),
            ("nature", vec!["rain", "thunder", "wind", "waves"]),
            (
                "vehicles",
                vec!["car_horn", "train", "airplane", "motorcycle"],
            ),
        ];

        let category_idx = self.rng.gen_range(0..categories.len());
        let (sound_category, available_sounds) = &categories[category_idx];

        // Number of sounds based on difficulty
        let sound_count = match difficulty {
            1..=3 => 2,
            4..=6 => 3,
            7..=8 => 4,
            _ => 5,
        };

        // Select random sounds
        let mut target_sounds = Vec::new();
        for _ in 0..sound_count.min(available_sounds.len() as u8) {
            let sound_idx = self.rng.gen_range(0..available_sounds.len());
            let sound = available_sounds[sound_idx].to_string();
            if !target_sounds.contains(&sound) {
                target_sounds.push(sound);
            }
        }

        let challenge_type = AudioChallengeType::SoundIdentification {
            sound_category: sound_category.to_string(),
            sound_count: target_sounds.len() as u8,
            target_sounds: target_sounds.clone(),
        };

        let answer = target_sounds.join(",");

        let time_limit = 60 + (sound_count as u32 * 10);

        Ok(AudioChallenge {
            challenge_type,
            instructions: format!("Identify the {} sounds you hear", sound_category),
            audio_url: format!("/api/v1/captcha/audio/sounds/{}", uuid::Uuid::new_v4()),
            answer,
            difficulty,
            time_limit,
            allow_replay: true,
            max_replays: 2,
        })
    }

    /// Generate a random audio challenge based on difficulty
    pub fn generate_random_challenge(
        &mut self,
        difficulty: u8,
    ) -> Result<AudioChallenge, CaptchaError> {
        let challenge_type = if difficulty <= 3 {
            // Easier challenges
            self.rng.gen_range(0..2) // Spoken digits or words
        } else if difficulty <= 6 {
            self.rng.gen_range(0..4) // Add tone sequence and sound identification
        } else {
            self.rng.gen_range(0..5) // All types including pattern recognition
        };

        match challenge_type {
            0 => self.generate_spoken_digits_challenge(difficulty),
            1 => self.generate_spoken_words_challenge(difficulty),
            2 => self.generate_tone_sequence_challenge(difficulty),
            3 => self.generate_sound_identification_challenge(difficulty),
            4 => self.generate_pattern_recognition_challenge(difficulty),
            _ => unreachable!(),
        }
    }
}

impl Default for AudioChallengeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tone_sequence_generation() {
        let mut generator = AudioChallengeGenerator::new();

        for difficulty in 1..=10 {
            let challenge = generator.generate_tone_sequence_challenge(difficulty);
            assert!(challenge.is_ok());

            let challenge = challenge.unwrap();
            assert_eq!(challenge.difficulty, difficulty);
            assert!(!challenge.answer.is_empty());
        }
    }

    #[test]
    fn test_spoken_digits_generation() {
        let mut generator = AudioChallengeGenerator::new();

        let challenge = generator.generate_spoken_digits_challenge(5).unwrap();
        assert_eq!(challenge.difficulty, 5);
        assert!(challenge.answer.len() >= 4);
        assert!(challenge.answer.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_spoken_words_generation() {
        let mut generator = AudioChallengeGenerator::new();

        for difficulty in 1..=10 {
            let challenge = generator.generate_spoken_words_challenge(difficulty);
            assert!(challenge.is_ok());

            let challenge = challenge.unwrap();
            assert!(!challenge.answer.is_empty());
        }
    }

    #[test]
    fn test_pattern_recognition_generation() {
        let mut generator = AudioChallengeGenerator::new();

        let challenge = generator.generate_pattern_recognition_challenge(7).unwrap();
        assert_eq!(challenge.difficulty, 7);
        assert!(
            ["ascending", "descending", "alternating", "repeating"]
                .contains(&challenge.answer.as_str())
        );
    }

    #[test]
    fn test_sound_identification_generation() {
        let mut generator = AudioChallengeGenerator::new();

        for difficulty in 1..=10 {
            let challenge = generator.generate_sound_identification_challenge(difficulty);
            assert!(challenge.is_ok());
        }
    }

    #[test]
    fn test_random_challenge_generation() {
        let mut generator = AudioChallengeGenerator::new();

        for _ in 0..10 {
            let challenge = generator.generate_random_challenge(5);
            assert!(challenge.is_ok());
        }
    }
}
