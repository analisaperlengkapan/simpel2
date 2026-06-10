use authenc_storage::Database;
use authenc_types::{
    AuthencError, Result,
    domain::captcha::{Challenge, ChallengeType},
};
use chrono::{DateTime, Utc};
use rand::{Rng, thread_rng};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::IpAddr;
use uuid::Uuid;

/// Core service for CAPTCHA operations
pub struct CaptchaService {
    db: Database,
}

impl CaptchaService {
    /// Create a new CaptchaService instance
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Generate a new CAPTCHA challenge, store it in the database, and return the metadata and JSON challenge data
    pub async fn generate_challenge(
        &self,
        challenge_type: ChallengeType,
        difficulty: u32,
        ip_address: IpAddr,
        session_id: Option<String>,
    ) -> Result<(Challenge, String)> {
        let challenge_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::minutes(5);

        let difficulty_clamped = difficulty.clamp(1, 10) as u8;

        // Generate challenge payload and raw answer
        let (challenge_data, answer, raw_data) = match challenge_type {
            ChallengeType::Logical => {
                let (json_data, ans) = generate_math_challenge(difficulty_clamped);
                (json_data.clone(), ans, json_data)
            }
            ChallengeType::Visual => {
                let (json_data, ans, svg) = generate_text_recognition_challenge(difficulty_clamped);
                (json_data, ans, svg)
            }
        };

        // Hash the answer using Sha256
        let mut hasher = Sha256::new();
        hasher.update(answer.as_bytes());
        let answer_hash = format!("{:x}", hasher.finalize());

        let challenge = Challenge {
            id: challenge_id,
            challenge_type,
            answer_hash,
            difficulty,
            expires_at,
            verified: false,
            session_id,
            created_at: now,
        };

        // Persist challenge to the database
        self.db
            .store_captcha_challenge(&challenge, ip_address, raw_data)
            .await?;

        Ok((challenge, challenge_data))
    }

    /// Verify a CAPTCHA challenge response and log the validation attempt
    pub async fn verify_challenge(
        &self,
        challenge_id: Uuid,
        answer: &str,
        ip_address: IpAddr,
        user_agent: Option<String>,
    ) -> Result<bool> {
        // Retrieve challenge from DB
        let challenge = self
            .db
            .get_captcha_challenge(challenge_id)
            .await?
            .ok_or_else(|| {
                AuthencError::validation("Tantangan tidak ditemukan atau sudah kedaluwarsa")
            })?;

        // 1. Expiry check
        if Utc::now() > challenge.expires_at {
            self.db
                .record_captcha_validation_attempt(
                    challenge_id,
                    ip_address,
                    user_agent,
                    answer.to_string(),
                    false,
                    Some(0.5),
                )
                .await?;
            return Err(AuthencError::validation("Tantangan telah kedaluwarsa"));
        }

        // 2. Reuse check
        if challenge.verified {
            self.db
                .record_captcha_validation_attempt(
                    challenge_id,
                    ip_address,
                    user_agent,
                    answer.to_string(),
                    false,
                    Some(0.7),
                )
                .await?;
            return Err(AuthencError::validation("Tantangan sudah pernah digunakan"));
        }

        // 3. Verify answer
        // Hash user's input answer using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(answer.trim().as_bytes());
        let input_hash = format!("{:x}", hasher.finalize());

        // For math / logical challenges, do a case-insensitive fallback if direct hash mismatch
        let is_correct = if challenge.answer_hash == input_hash {
            true
        } else if challenge.challenge_type == ChallengeType::Logical {
            // Check case-insensitive trim
            let mut hasher_ci = Sha256::new();
            hasher_ci.update(answer.trim().to_lowercase().as_bytes());
            let input_hash_ci = format!("{:x}", hasher_ci.finalize());
            challenge.answer_hash == input_hash_ci
        } else {
            false
        };

        // Record verification attempt
        self.db
            .record_captcha_validation_attempt(
                challenge_id,
                ip_address,
                user_agent,
                answer.to_string(),
                is_correct,
                if is_correct { Some(0.1) } else { Some(0.6) },
            )
            .await?;

        if is_correct {
            // Update challenge status to verified
            self.db
                .mark_captcha_challenge_verified(challenge_id)
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Redeem a previously solved challenge as a single-use login token (#49).
    ///
    /// The login handler calls this when brute-force protection requires a
    /// CAPTCHA: the `captcha_token` returned by `POST /captcha/verify` IS the
    /// solved challenge id, and this consumes it atomically (single-use, bounded
    /// by the challenge's expiry). Returns `true` iff the challenge was solved,
    /// unexpired, and not already consumed.
    pub async fn redeem_solved(&self, challenge_id: Uuid) -> Result<bool> {
        self.db.consume_solved_captcha(challenge_id).await
    }
}

// ============================================
// Challenge Generators (Centralized)
// ============================================

/// Generate a math challenge (e.g., "What is 7 + 3?")
fn generate_math_challenge(difficulty: u8) -> (String, String) {
    let mut rng = thread_rng();

    let (a, b, op, answer): (i32, i32, &str, String) = match difficulty {
        1..=3 => {
            let a: i32 = rng.gen_range(1..=20);
            let b: i32 = rng.gen_range(1..=20);
            if rng.gen_bool(0.5) {
                (a, b, "+", (a + b).to_string())
            } else {
                let (a, b) = if a >= b { (a, b) } else { (b, a) };
                (a, b, "-", (a - b).to_string())
            }
        }
        4..=6 => {
            let a: i32 = rng.gen_range(2..=12);
            let b: i32 = rng.gen_range(2..=12);
            (a, b, "×", (a * b).to_string())
        }
        _ => {
            let a: i32 = rng.gen_range(10..=50);
            let b: i32 = rng.gen_range(2..=20);
            if rng.gen_bool(0.5) {
                (a, b, "+", (a + b).to_string())
            } else {
                (a, b, "×", (a * b).to_string())
            }
        }
    };

    let correct: i32 = answer.parse::<i32>().unwrap_or(0);
    let mut options: Vec<i32> = vec![correct];
    while options.len() < 4 {
        let offset: i32 = rng.gen_range(1..=5) * if rng.gen_bool(0.5) { 1 } else { -1 };
        let wrong = correct + offset;
        if wrong >= 0 && !options.contains(&wrong) {
            options.push(wrong);
        }
    }

    for i in (1..options.len()).rev() {
        let j = rng.gen_range(0..=i);
        options.swap(i, j);
    }

    let visual = format!("{} {} {}", a, op, b);
    let instructions = format!("Berapa hasil dari {} {} {}?", a, op, b);
    let options_json: Vec<serde_json::Value> = options.iter().map(|o| json!(*o)).collect();

    let challenge_data = json!({
        "challenge_type": "math",
        "instructions": instructions,
        "visual": visual,
        "options": options_json,
        "data": format!("{}:{}:{}", visual, op, answer),
    })
    .to_string();

    (challenge_data, answer)
}

/// Generate a text recognition challenge with server-side SVG rendering
fn generate_text_recognition_challenge(difficulty: u8) -> (String, String, String) {
    let mut rng = thread_rng();

    let length: usize = match difficulty {
        1..=3 => 5,
        4..=6 => 6,
        _ => 7,
    };

    let uppercase: Vec<char> = "ABCDEFGHJKLMNPQRTUVWXYZ".chars().collect();
    let lowercase: Vec<char> = "abcdefghjkmnpqrstuvwxyz".chars().collect();
    let digits: Vec<char> = "2346789".chars().collect();

    let mut text_chars: Vec<char> = Vec::with_capacity(length);
    text_chars.push(uppercase[rng.gen_range(0..uppercase.len())]);
    text_chars.push(lowercase[rng.gen_range(0..lowercase.len())]);
    text_chars.push(digits[rng.gen_range(0..digits.len())]);

    let all_chars: Vec<char> = uppercase
        .iter()
        .chain(lowercase.iter())
        .chain(digits.iter())
        .copied()
        .collect();
    for _ in 3..length {
        text_chars.push(all_chars[rng.gen_range(0..all_chars.len())]);
    }

    for i in (1..text_chars.len()).rev() {
        let j = rng.gen_range(0..=i);
        text_chars.swap(i, j);
    }

    let text: String = text_chars.iter().collect();
    let svg = generate_captcha_svg(&text, &mut rng, difficulty);
    let nonce = Uuid::new_v4().to_string();

    let challenge_data = json!({
        "challenge_type": "text_recognition",
        "instructions": "Ketik karakter yang ditampilkan di bawah ini",
        "svg": svg,
        "data": format!("challenge:{}", nonce),
    })
    .to_string();

    (challenge_data, text, svg)
}

fn xml_escape_char(ch: char) -> String {
    match ch {
        '&' => "&amp;".to_string(),
        '<' => "&lt;".to_string(),
        '>' => "&gt;".to_string(),
        '"' => "&quot;".to_string(),
        '\'' => "&#x27;".to_string(),
        _ => ch.to_string(),
    }
}

fn generate_captcha_svg(text: &str, rng: &mut impl Rng, difficulty: u8) -> String {
    let char_count = text.chars().count();
    let vb_width = 50 * char_count + 30;
    let vb_height = 70;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="100%" preserveAspectRatio="xMidYMid meet" viewBox="0 0 {vb_width} {vb_height}" style="display:block;border-radius:8px;">"#,
    );

    let width = vb_width;
    let height = vb_height;

    svg.push_str(&format!(
        r##"<rect width="{width}" height="{height}" fill="#1a237e" rx="8"/>"##,
    ));

    let dot_count = 15 + (difficulty as usize) * 5;
    for _ in 0..dot_count {
        let cx = rng.gen_range(0..width);
        let cy = rng.gen_range(0..height);
        let r: f64 = rng.gen_range(1.0..3.5);
        let opacity: f64 = rng.gen_range(0.1..0.35);
        let colors = ["#5c6bc0", "#7986cb", "#9fa8da", "#c5cae9", "#3949ab"];
        let color = colors[rng.gen_range(0..colors.len())];
        svg.push_str(&format!(
            r#"<circle cx="{cx}" cy="{cy}" r="{r:.1}" fill="{color}" opacity="{opacity:.2}"/>"#,
        ));
    }

    let line_count = 4 + (difficulty as usize) * 2;
    for _ in 0..line_count {
        let x1 = rng.gen_range(0..width);
        let y1 = rng.gen_range(0..height);
        let x2 = rng.gen_range(0..width);
        let y2 = rng.gen_range(0..height);
        let sw: f64 = rng.gen_range(1.0..2.5);
        let opacity: f64 = rng.gen_range(0.3..0.7);
        let colors = [
            "#e8eaf6", "#c5cae9", "#9fa8da", "#7986cb", "#5c6bc0", "#3f51b5",
        ];
        let color = colors[rng.gen_range(0..colors.len())];
        svg.push_str(&format!(
            r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{color}" stroke-width="{sw:.1}" opacity="{opacity:.2}"/>"#,
        ));
    }

    let arc_count = 2 + (difficulty as usize);
    for _ in 0..arc_count {
        let sx = rng.gen_range(0..(width / 4));
        let sy = rng.gen_range(10..height - 10);
        let cx1 = rng.gen_range(width / 4..width / 2) as i32;
        let cy1 = rng.gen_range(0..height);
        let cx2 = rng.gen_range(width / 2..3 * width / 4) as i32;
        let cy2 = rng.gen_range(0..height);
        let ex = rng.gen_range(3 * width / 4..width);
        let ey = rng.gen_range(10..height - 10);
        let sw: f64 = rng.gen_range(1.0..2.5);
        let opacity: f64 = rng.gen_range(0.25..0.55);
        svg.push_str(&format!(
            r##"<path d="M{sx},{sy} C{cx1},{cy1} {cx2},{cy2} {ex},{ey}" fill="none" stroke="#9fa8da" stroke-width="{sw:.1}" opacity="{opacity:.2}"/>"##,
        ));
    }

    let padding = 20.0;
    let usable = width as f64 - padding * 2.0;
    let char_spacing = usable / char_count as f64;
    for (i, ch) in text.chars().enumerate() {
        let x = padding + (i as f64 + 0.5) * char_spacing + rng.gen_range(-3.0..3.0);
        let y = (height as f64) / 2.0 + rng.gen_range(-6.0..6.0);
        let rotation: f64 = rng.gen_range(-20.0..20.0);
        let font_size = rng.gen_range(24..32);
        let scale_x: f64 = rng.gen_range(0.9..1.1);
        let scale_y: f64 = rng.gen_range(0.9..1.1);

        let char_colors = [
            "#e8eaf6", "#c5cae9", "#ffffff", "#bbdefb", "#d1c4e9", "#f3e5f5",
        ];
        let color = char_colors[rng.gen_range(0..char_colors.len())];

        let weights = ["bold", "800", "900"];
        let weight = weights[rng.gen_range(0..weights.len())];

        let safe_ch = xml_escape_char(ch);
        svg.push_str(&format!(
            r#"<g transform="translate({x:.1},{y:.1})"><text font-family="monospace,Courier,serif" font-size="{font_size}" font-weight="{weight}" fill="{color}" transform="rotate({rotation:.1}) scale({scale_x:.2},{scale_y:.2})" text-anchor="middle" dominant-baseline="middle">{safe_ch}</text></g>"#,
        ));
    }

    let scratch_count = 3 + (difficulty as usize);
    for _ in 0..scratch_count {
        let x1 = rng.gen_range(10..width - 10);
        let y1 = rng.gen_range(20..height - 20);
        let x2 = x1 as i32 + rng.gen_range(-60..60);
        let y2 = y1 + rng.gen_range(-15..15);
        let sw: f64 = rng.gen_range(0.8..2.0);
        let opacity: f64 = rng.gen_range(0.3..0.65);
        svg.push_str(&format!(
            r##"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="#b0bec5" stroke-width="{sw:.1}" opacity="{opacity:.2}" stroke-linecap="round"/>"##,
        ));
    }

    svg.push_str("</svg>");
    svg
}
