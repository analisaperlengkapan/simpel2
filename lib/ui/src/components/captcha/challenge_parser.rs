//! Challenge Data Parser
//!
//! Utilities for parsing CAPTCHA challenge JSON data

use super::types::ChallengeResponse;
use serde_json::Value;

/// Parse the question text from challenge data
pub fn parse_challenge_question(data: Option<ChallengeResponse>) -> String {
    data.and_then(|d| {
        serde_json::from_str::<Value>(&d.challenge_data)
            .ok()
            .and_then(|v| {
                v.get("question")
                    .and_then(|q| q.as_str())
                    .map(|s| s.to_string())
            })
    })
    .unwrap_or_else(|| "Loading...".to_string())
}

/// Parse the visual display text from challenge data
pub fn parse_challenge_visual(data: Option<ChallengeResponse>) -> String {
    data.and_then(|d| {
        serde_json::from_str::<Value>(&d.challenge_data)
            .ok()
            .and_then(|v| {
                v.get("visual")
                    .and_then(|visual| visual.as_str().map(|s| s.to_string()))
                    .or_else(|| {
                        // Fallback to question if no visual field
                        v.get("question")
                            .and_then(|q| q.as_str())
                            .map(|s| s.to_string())
                    })
            })
    })
    .unwrap_or_else(|| "...".to_string())
}

/// Parse answer options from challenge data
pub fn parse_answer_options(data: Option<ChallengeResponse>) -> Vec<String> {
    data.and_then(|d| {
        serde_json::from_str::<Value>(&d.challenge_data)
            .ok()
            .and_then(|v| {
                v.get("options").and_then(|opts| {
                    opts.as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|val| {
                                val.as_u64()
                                    .map(|n| n.to_string())
                                    .or_else(|| val.as_str().map(|s| s.to_string()))
                            })
                            .collect()
                    })
                })
            })
    })
    .unwrap_or_else(|| vec!["...".to_string()])
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
