//! Real-time Validation Feedback Components
//!
//! Provides immediate feedback on CAPTCHA validation attempts

use super::types::*;
use crate::components::forms::Button;
use crate::core::types::{ButtonSize, ButtonVariant};
use leptos::prelude::*;

/// Validation status for real-time feedback
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStatus {
    Idle,
    Validating,
    Success,
    Failed(String),
    RateLimited(u32), // seconds until retry
}

/// Progressive difficulty indicator component
#[component]
pub fn DifficultyIndicator(
    current_difficulty: u8,
    max_difficulty: u8,
    attempts: u32,
    max_attempts: u32,
) -> impl IntoView {
    let difficulty_percentage = (current_difficulty as f32 / max_difficulty as f32 * 100.0) as u32;
    let _ = (attempts as f32 / max_attempts as f32 * 100.0) as u32;

    view! {
        <div class="difficulty-indicator bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-3 mb-4">
            <div class="flex items-center justify-between mb-2">
                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                    "Challenge Difficulty"
                </span>
                <span class="text-xs text-gray-500 dark:text-gray-400">
                    "Level " {current_difficulty} "/" {max_difficulty}
                </span>
            </div>

            <div class="difficulty-bar bg-gray-200 dark:bg-gray-700 rounded-full h-2 mb-3">
                <div
                    class=move || format!(
                        "h-2 rounded-full transition-all duration-500 {}",
                        match current_difficulty {
                            1..=3 => "bg-green-500",
                            4..=6 => "bg-yellow-500",
                            7..=8 => "bg-orange-500",
                            _ => "bg-red-500",
                        }
                    )
                    style=move || format!("width: {}%", difficulty_percentage)
                ></div>
            </div>

            <div class="flex items-center justify-between text-xs">
                <div class="flex items-center space-x-2">
                    <span class="text-gray-600 dark:text-gray-400">"Attempts:"</span>
                    <div class="flex space-x-1">
                        {(0..max_attempts).map(|i| {
                            view! {
                                <div class=format!(
                                    "w-2 h-2 rounded-full {}",
                                    if i < attempts { "bg-red-400" } else { "bg-gray-300 dark:bg-gray-600" }
                                )></div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                <span class=move || format!(
                    "font-medium {}",
                    match current_difficulty {
                        1..=3 => "text-green-600 dark:text-green-400",
                        4..=6 => "text-yellow-600 dark:text-yellow-400",
                        7..=8 => "text-orange-600 dark:text-orange-400",
                        _ => "text-red-600 dark:text-red-400",
                    }
                )>
                    {match current_difficulty {
                        1..=3 => "Easy",
                        4..=6 => "Medium",
                        7..=8 => "Hard",
                        _ => "Expert",
                    }}
                </span>
            </div>
        </div>
    }
}

/// Real-time validation status component
#[component]
pub fn ValidationStatusIndicator(
    status: ReadSignal<ValidationStatus>,
    on_retry: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="validation-status">
            {move || {
                match status.get() {
                    ValidationStatus::Idle => {
                        let _: () = view! {};
                        ().into_any()
                    }

                    ValidationStatus::Validating => view! {
                        <div class="validating-indicator bg-blue-50 dark:bg-blue-900 border border-blue-200 dark:border-blue-700 rounded-lg p-3 mb-4">
                            <div class="flex items-center space-x-3">
                                <div class="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-600"></div>
                                <div>
                                    <div class="text-sm font-medium text-blue-900 dark:text-blue-100">
                                        "Validating your response..."
                                    </div>
                                    <div class="text-xs text-blue-700 dark:text-blue-300">
                                        "Analyzing behavioral patterns and answer"
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any(),

                    ValidationStatus::Success => view! {
                        <div class="success-indicator bg-green-50 dark:bg-green-900 border border-green-200 dark:border-green-700 rounded-lg p-3 mb-4">
                            <div class="flex items-center space-x-3">
                                <div class="flex-shrink-0">
                                    <div class="w-5 h-5 bg-green-500 rounded-full flex items-center justify-center">
                                        <span class="text-white text-xs">"✓"</span>
                                    </div>
                                </div>
                                <div>
                                    <div class="text-sm font-medium text-green-900 dark:text-green-100">
                                        "Verification successful!"
                                    </div>
                                    <div class="text-xs text-green-700 dark:text-green-300">
                                        "You have been verified as human"
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any(),

                    ValidationStatus::Failed(message) => view! {
                        <div class="failed-indicator bg-red-50 dark:bg-red-900 border border-red-200 dark:border-red-700 rounded-lg p-3 mb-4">
                            <div class="flex items-start space-x-3">
                                <div class="flex-shrink-0">
                                    <div class="w-5 h-5 bg-red-500 rounded-full flex items-center justify-center">
                                        <span class="text-white text-xs">"✕"</span>
                                    </div>
                                </div>
                                <div class="flex-1">
                                    <div class="text-sm font-medium text-red-900 dark:text-red-100">
                                        "Verification failed"
                                    </div>
                                    <div class="text-xs text-red-700 dark:text-red-300 mb-2">
                                        {message}
                                    </div>
                                    {if let Some(_retry_callback) = on_retry {
                                        view! {
                                            <Button
                                                // on_click=Some(Box::new(move || _retry_callback(())))
                                                variant=ButtonVariant::Danger
                                                size=ButtonSize::Small
                                            >
                                                "Try Again"
                                            </Button>
                                        }.into_any()
                                    } else {
                                        let _: () = view! {};
                                        ().into_any()
                                    }}
                                </div>
                            </div>
                        </div>
                    }.into_any(),

                    ValidationStatus::RateLimited(seconds) => view! {
                        <div class="rate-limited-indicator bg-yellow-50 dark:bg-yellow-900 border border-yellow-200 dark:border-yellow-700 rounded-lg p-3 mb-4">
                            <div class="flex items-start space-x-3">
                                <div class="flex-shrink-0">
                                    <div class="w-5 h-5 bg-yellow-500 rounded-full flex items-center justify-center">
                                        <span class="text-white text-xs">"⏱"</span>
                                    </div>
                                </div>
                                <div>
                                    <div class="text-sm font-medium text-yellow-900 dark:text-yellow-100">
                                        "Too many attempts"
                                    </div>
                                    <div class="text-xs text-yellow-700 dark:text-yellow-300">
                                        "Please wait " {seconds} " seconds before trying again"
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                }
            }}
        </div>
    }
}

/// Input validation feedback component
#[component]
pub fn InputValidationFeedback(
    input_value: ReadSignal<String>,
    challenge_type: ChallengeType,
    is_validating: bool,
) -> impl IntoView {
    // Real-time input validation
    let validation_state = move || {
        let value = input_value.get();
        let trimmed_value = value.trim();

        if trimmed_value.is_empty() {
            return ("idle", "Enter your answer", "text-gray-500");
        }

        match challenge_type {
            ChallengeType::Visual | ChallengeType::Logical => {
                if value.chars().all(|c| c.is_ascii_digit()) {
                    ("valid", "Looks good!", "text-green-600")
                } else {
                    ("warning", "Expected a number", "text-yellow-600")
                }
            }
            ChallengeType::Audio => {
                if value.len() >= 2 {
                    ("valid", "Answer received", "text-green-600")
                } else {
                    ("partial", "Keep typing...", "text-blue-600")
                }
            }
            _ => {
                if !value.is_empty() {
                    ("valid", "Answer received", "text-green-600")
                } else {
                    ("idle", "Enter your answer", "text-gray-500")
                }
            }
        }
    };

    view! {
        <div class="input-feedback mt-1">
            {move || {
                let (state, message, color_class) = validation_state();

                view! {
                    <div class=format!("flex items-center space-x-2 text-xs {}", color_class)>
                        <span class=format!(
                            "w-2 h-2 rounded-full {}",
                            match state {
                                "valid" => "bg-green-500",
                                "warning" => "bg-yellow-500",
                                "partial" => "bg-blue-500 animate-pulse",
                                _ => "bg-gray-300",
                            }
                        )></span>
                        <span>{message}</span>
                        {if is_validating {
                            view! {
                                <div class="animate-spin rounded-full h-3 w-3 border border-gray-400 border-t-transparent"></div>
                            }.into_any()
                        } else {
                            let _: () = view! {};
                            ().into_any()
                        }}
                    </div>
                }
            }}
        </div>
    }
}

/// Progress indicator for multi-step challenges
#[component]
pub fn ChallengeProgressIndicator(
    current_step: u8,
    total_steps: u8,
    step_names: Vec<String>,
) -> impl IntoView {
    view! {
        <div class="challenge-progress mb-4">
            <div class="flex items-center justify-between mb-2">
                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                    "Challenge Progress"
                </span>
                <span class="text-xs text-gray-500 dark:text-gray-400">
                    "Step " {current_step} " of " {total_steps}
                </span>
            </div>

            <div class="flex items-center space-x-2">
                {(1..=total_steps).map(|step| {
                    let is_current = step == current_step;
                    let is_completed = step < current_step;
                    let _ = step_names.get((step - 1) as usize).cloned().unwrap_or_else(|| format!("Step {}", step));

                    view! {
                        <div class="flex items-center">
                            <div class=format!(
                                "w-8 h-8 rounded-full flex items-center justify-center text-xs font-medium transition-colors {}",
                                if is_completed {
                                    "bg-green-500 text-white"
                                } else if is_current {
                                    "bg-blue-500 text-white"
                                } else {
                                    "bg-gray-300 dark:bg-gray-600 text-gray-600 dark:text-gray-400"
                                }
                            )>
                                {if is_completed { "✓".to_string() } else { format!("{}", step) }}
                            </div>

                            {if step < total_steps {
                                view! {
                                    <div class=format!(
                                        "w-8 h-0.5 mx-1 {}",
                                        if is_completed { "bg-green-500" } else { "bg-gray-300 dark:bg-gray-600" }
                                    )></div>
                                }.into_any()
                            } else {
                                let _: () = view! {};
                                ().into_any()
                            }}
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="mt-2 text-xs text-gray-600 dark:text-gray-400 text-center">
                {step_names.get((current_step - 1) as usize).cloned().unwrap_or_else(|| format!("Step {}", current_step))}
            </div>
        </div>
    }
}

/// Retry mechanism component
#[component]
pub fn RetryMechanism(
    attempts_remaining: u8,
    max_attempts: u8,
    cooldown_seconds: Option<u32>,
    on_retry: Callback<()>,
    on_new_challenge: Callback<()>,
) -> impl IntoView {
    let _ = max_attempts;
    let _ = on_retry;
    let _ = on_new_challenge;

    let (countdown, set_countdown) = signal(cooldown_seconds.unwrap_or(0));

    // Countdown effect
    Effect::new(move |_| {
        if let Some(initial_seconds) = cooldown_seconds {
            set_countdown.set(initial_seconds);

            let _ = gloo_timers::callback::Interval::new(1000, move || {
                set_countdown.update(|count| {
                    if *count > 0 {
                        *count -= 1;
                    }
                });
            });

            // on_cleanup(move || {
            //     drop(interval_handle);
            // });
        }
    });

    view! {
        <div class="retry-mechanism bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
            <div class="text-center">
                <div class="text-sm font-medium text-gray-900 dark:text-gray-100 mb-2">
                    {if attempts_remaining > 0 {
                        format!("You have {} attempt{} remaining", attempts_remaining, if attempts_remaining == 1 { "" } else { "s" })
                    } else {
                        "No attempts remaining".to_string()
                    }}
                </div>

                {if countdown.get() > 0 {
                    view! {
                        <div class="text-xs text-gray-600 dark:text-gray-400 mb-3">
                            "Please wait " {countdown.get()} " seconds before retrying"
                        </div>
                    }.into_any()
                } else {
                    let _: () = view! {};
                    ().into_any()
                }}

                <div class="flex justify-center space-x-2">
                    {if attempts_remaining > 0 && countdown.get() == 0 {
                        view! {
                            <Button
                                // on_click=Some(Box::new(move || on_retry(())))
                                variant=ButtonVariant::Primary
                                size=ButtonSize::Small
                            >
                                "Retry Challenge"
                            </Button>
                        }.into_any()
                    } else {
                        let _: () = view! {};
                        ().into_any()
                    }}

                    <Button
                        // on_click=Some(Box::new(move || on_new_challenge(())))
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Small
                        disabled=countdown.get() != 0
                    >
                        "New Challenge"
                    </Button>
                </div>
            </div>
        </div>
    }
}
