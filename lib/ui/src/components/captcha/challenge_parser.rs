//! Challenge Data Parser
//!
//! Utilities for parsing CAPTCHA challenge JSON data

use super::types::ChallengeResponse;
use serde_json::Value;

/// Parsed challenge data for rendering
#[derive(Clone, Debug, Default)]
pub struct ParsedChallenge {
    /// The challenge type (e.g., "text_recognition", "image_selection")
    pub challenge_type: String,
    /// Instructions for the user
    pub instructions: String,
    /// Visual or display data
    pub display_data: String,
    /// Answer options if available
    pub options: Vec<String>,
    /// Images for image selection challenges
    pub images: Vec<ImageData>,
    /// Grid size for image grid
    pub grid_size: usize,
    /// Text to recognize (for text_recognition)
    pub text_to_recognize: Option<String>,
}

/// Image data for image selection challenges
#[derive(Clone, Debug, Default)]
pub struct ImageData {
    pub index: usize,
    pub url: String,
    pub alt: String,
}

/// Parse challenge data into structured format
pub fn parse_challenge(data: Option<ChallengeResponse>) -> ParsedChallenge {
    let Some(response) = data else {
        return ParsedChallenge::default();
    };

    // Parse outer JSON
    let Ok(outer) = serde_json::from_str::<Value>(&response.challenge_data) else {
        return ParsedChallenge::default();
    };

    let challenge_type = outer
        .get("challenge_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let instructions = outer
        .get("instructions")
        .and_then(|v| v.as_str())
        .unwrap_or("Complete the challenge")
        .to_string();

    // Parse inner data field (may be nested JSON string)
    let inner_data = outer.get("data").and_then(|v| {
        v.as_str()
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .or_else(|| Some(v.clone()))
    });

    let mut parsed = ParsedChallenge {
        challenge_type: challenge_type.clone(),
        instructions,
        ..Default::default()
    };

    match challenge_type.as_str() {
        "text_recognition" => {
            // Extract text to recognize from data field
            // Format: "VAZF:4f668449-2db3-4df5-88ec-946c9c520fa1"
            if let Some(data_str) = outer.get("data").and_then(|v| v.as_str()) {
                if let Some((text, _)) = data_str.split_once(':') {
                    parsed.text_to_recognize = Some(text.to_string());
                    parsed.display_data = text.to_string();
                }
            }
        }
        "image_selection" => {
            if let Some(inner) = inner_data {
                parsed.grid_size =
                    inner.get("grid_size").and_then(|v| v.as_u64()).unwrap_or(3) as usize;

                let total_images = inner
                    .get("total_images")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(9) as usize;

                let category = inner
                    .get("category")
                    .and_then(|v| v.as_str())
                    .unwrap_or("objects");

                let nonce = inner
                    .get("nonce")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");

                // Generate placeholder images (backend should provide actual URLs)
                for i in 0..total_images {
                    parsed.images.push(ImageData {
                        index: i,
                        url: format!("/api/captcha/image/{}/{}", nonce, i),
                        alt: format!("Image {} for {} selection", i + 1, category),
                    });
                }
            }
        }
        "math" | "logical" => {
            // Legacy format support
            parsed.display_data = outer
                .get("visual")
                .or_else(|| outer.get("question"))
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();

            if let Some(opts) = outer.get("options").and_then(|v| v.as_array()) {
                parsed.options = opts
                    .iter()
                    .filter_map(|v| {
                        v.as_u64()
                            .map(|n| n.to_string())
                            .or_else(|| v.as_str().map(|s| s.to_string()))
                    })
                    .collect();
            }
        }
        _ => {
            // Fallback
            parsed.display_data = outer
                .get("data")
                .and_then(|v| v.as_str())
                .unwrap_or("...")
                .to_string();
        }
    }

    parsed
}

/// Parse the question text from challenge data (legacy support)
pub fn parse_challenge_question(data: Option<ChallengeResponse>) -> String {
    let parsed = parse_challenge(data);
    if !parsed.instructions.is_empty() {
        parsed.instructions
    } else {
        "Complete the challenge".to_string()
    }
}

/// Parse the visual display text from challenge data (legacy support)
pub fn parse_challenge_visual(data: Option<ChallengeResponse>) -> String {
    let parsed = parse_challenge(data);
    if let Some(text) = parsed.text_to_recognize {
        text
    } else if !parsed.display_data.is_empty() {
        parsed.display_data
    } else {
        "...".to_string()
    }
}

/// Parse answer options from challenge data
pub fn parse_answer_options(data: Option<ChallengeResponse>) -> Vec<String> {
    let parsed = parse_challenge(data);
    if !parsed.options.is_empty() {
        parsed.options
    } else {
        vec!["...".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_challenge_question() {
        let response = ChallengeResponse {
            challenge_id: "test-id".to_string(),
            challenge_type: crate::components::captcha::types::ChallengeType::Visual,
            challenge_data: r#"{"type":"math","question":"What is 7 + 3?","visual":"7 + 3","options":[10,8,11,9]}"#.to_string(),
            difficulty: 3,
            expires_at: 1234567890,
            metadata: None,
        };

        let question = parse_challenge_question(Some(response.clone()));
        assert_eq!(question, "What is 7 + 3?");

        let visual = parse_challenge_visual(Some(response.clone()));
        assert_eq!(visual, "7 + 3");

        let options = parse_answer_options(Some(response));
        assert_eq!(options, vec!["10", "8", "11", "9"]);
    }

    #[test]
    fn test_parse_missing_data() {
        let question = parse_challenge_question(None);
        assert_eq!(question, "Loading...");

        let visual = parse_challenge_visual(None);
        assert_eq!(visual, "...");

        let options = parse_answer_options(None);
        assert_eq!(options, vec!["..."]);
    }
}
