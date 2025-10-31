//! Test module for challenge generator
//!
//! Basic tests to verify challenge generation functionality

#[cfg(test)]
mod tests {
    use super::super::generator::*;
    use super::super::types::*;

    #[tokio::test]
    async fn test_visual_challenge_generation() {
        let generator = ChallengeGenerator::new();

        // Test different difficulty levels
        for difficulty in 1..=10 {
            let result = generator.generate_visual_challenge(difficulty).await;
            assert!(
                result.is_ok(),
                "Failed to generate visual challenge for difficulty {}",
                difficulty
            );

            let challenge_data = result.unwrap();
            assert!(
                !challenge_data.is_empty(),
                "Challenge data should not be empty"
            );

            // Verify it's valid JSON
            let parsed: serde_json::Value =
                serde_json::from_str(&challenge_data).expect("Challenge data should be valid JSON");
            assert!(parsed.is_object(), "Challenge data should be a JSON object");
        }
    }

    #[tokio::test]
    async fn test_logical_challenge_generation() {
        let generator = ChallengeGenerator::new();

        // Test different difficulty levels
        for difficulty in 1..=10 {
            let result = generator.generate_logical_challenge(difficulty).await;
            assert!(
                result.is_ok(),
                "Failed to generate logical challenge for difficulty {}",
                difficulty
            );

            let challenge_data = result.unwrap();
            assert!(
                !challenge_data.is_empty(),
                "Challenge data should not be empty"
            );

            // Verify it's valid JSON
            let parsed: serde_json::Value =
                serde_json::from_str(&challenge_data).expect("Challenge data should be valid JSON");
            assert!(parsed.is_object(), "Challenge data should be a JSON object");
        }
    }

    #[tokio::test]
    async fn test_complete_challenge_generation() {
        let generator = ChallengeGenerator::new();

        // Test all challenge types
        let challenge_types = vec![
            ChallengeType::Visual,
            ChallengeType::Logical,
            ChallengeType::Audio,
            ChallengeType::Behavioral,
            ChallengeType::Hybrid,
        ];

        for challenge_type in challenge_types {
            let result = generator
                .generate_challenge(
                    challenge_type.clone(),
                    5, // Medium difficulty
                    Some("test_session".to_string()),
                )
                .await;

            assert!(
                result.is_ok(),
                "Failed to generate {:?} challenge",
                challenge_type
            );

            let challenge = result.unwrap();
            assert_eq!(challenge.challenge_type, challenge_type);
            assert_eq!(challenge.difficulty_level, 5);
            assert!(!challenge.id.is_empty());
            assert!(!challenge.expected_answer_hash.is_empty());
        }
    }

    #[tokio::test]
    async fn test_invalid_difficulty_levels() {
        let generator = ChallengeGenerator::new();

        // Test invalid difficulty levels
        let invalid_difficulties = vec![0, 11, 255];

        for difficulty in invalid_difficulties {
            let result = generator
                .generate_challenge(ChallengeType::Visual, difficulty, None)
                .await;

            assert!(
                result.is_err(),
                "Should fail for invalid difficulty {}",
                difficulty
            );
        }
    }

    #[tokio::test]
    async fn test_challenge_uniqueness() {
        let generator = ChallengeGenerator::new();

        // Generate multiple challenges and verify they're unique
        let mut challenge_ids = std::collections::HashSet::new();
        let mut challenge_data = std::collections::HashSet::new();

        for _ in 0..10 {
            let challenge = generator
                .generate_challenge(ChallengeType::Visual, 5, None)
                .await
                .unwrap();

            // IDs should be unique
            assert!(
                challenge_ids.insert(challenge.id.clone()),
                "Challenge IDs should be unique"
            );

            // Challenge data should be unique (high probability)
            challenge_data.insert(challenge.encrypted_data.clone());
        }

        // We should have generated unique challenges
        assert!(
            challenge_data.len() > 1,
            "Challenges should have different data"
        );
    }
}
