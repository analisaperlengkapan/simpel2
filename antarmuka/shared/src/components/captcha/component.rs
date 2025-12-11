//! CAPTCHA Component Implementation
//!
//! Main CAPTCHA component using Leptos and shared component library

use super::accessibility::*;
use super::types::*;
use super::validation_feedback::*;
use crate::components::feedback::Alert;
use crate::components::forms::Button;
use crate::core::types::{AlertVariant, ButtonSize, ButtonVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Main CAPTCHA component
#[component]
pub fn Captcha(
    /// Callback when CAPTCHA is successfully completed
    #[prop(into)]
    on_success: Callback<String>,
    /// Callback when CAPTCHA fails
    #[prop(optional, into)]
    on_failure: Option<Callback<String>>,
    /// Initial difficulty level (1-10)
    #[prop(optional)]
    difficulty: Option<u8>,
    /// Enable accessibility features
    #[prop(default = true)]
    accessibility_enabled: bool,
    /// Custom CSS classes
    #[prop(optional, into)]
    class: Option<String>,
    /// Enable behavioral analysis
    #[prop(default = true)]
    behavioral_analysis: bool,
) -> impl IntoView {
    let (state, set_state) = signal(CaptchaState::default());
    let (challenge_data, set_challenge_data) = signal(None::<ChallengeResponse>);

    // Accessibility state
    let (current_focus, set_current_focus) = signal(None::<String>);
    let (announcements, set_announcements) = signal(Vec::<String>::new());
    let (show_alternative_inputs, set_show_alternative_inputs) = signal(false);

    // Behavioral analysis state
    let (behavioral_metrics, set_behavioral_metrics) =
        signal(None::<super::behavioral_tracker::SimpleBehavioralMetrics>);
    let session_id = format!("captcha_session_{}", js_sys::Date::now() as u64);

    // Validation feedback state
    let (validation_status, set_validation_status) =
        signal(super::validation_feedback::ValidationStatus::Idle);
    let (input_value, set_input_value) = signal(String::new());

    // Handle behavioral data updates
    let handle_behavioral_update =
        move |metrics: super::behavioral_tracker::SimpleBehavioralMetrics| {
            set_behavioral_metrics.set(Some(metrics));
        };

    // Initialize with provided difficulty
    if let Some(diff) = difficulty {
        set_state.update(|s| s.difficulty = diff);
    }

    // Clone session_id for use in async closures
    let session_id_clone = session_id.clone();

    // Challenge generation effect - wrap in Resource for reactive updates
    let (refresh_trigger, set_refresh_trigger) = signal(0);

    Effect::new(move |_| {
        // Track refresh trigger
        let _ = refresh_trigger.get();

        set_state.update(|s| {
            s.loading = true;
            s.error = None;
        });

        let session_id_for_spawn = session_id_clone.clone();

        spawn_local(async move {
            // Get Authenc URL from environment or use default
            let authenc_url = option_env!("AUTHENC_URL")
                .unwrap_or("http://localhost:8080")
                .to_string();

            let challenge_url = format!("{}/captcha/challenge", authenc_url);

            // Prepare request payload
            let request_payload = serde_json::json!({
                "challenge_type": "Visual",
                "difficulty": state.get().difficulty,
                "session_id": session_id_for_spawn,
            });

            // Make API call to Authenc
            match web_sys::window() {
                Some(window) => {
                    use wasm_bindgen::{JsCast, JsValue};
                    use web_sys::{Request, RequestInit, RequestMode, Response};

                    let opts = RequestInit::new();
                    opts.set_method("POST");
                    opts.set_mode(RequestMode::Cors);

                    // Set body
                    if let Ok(body_str) = serde_json::to_string(&request_payload) {
                        opts.set_body(&JsValue::from_str(&body_str));
                    }

                    match Request::new_with_str_and_init(&challenge_url, &opts) {
                        Ok(request) => {
                            // Set headers
                            let _ = request.headers().set("Content-Type", "application/json");

                            // Fetch challenge
                            match wasm_bindgen_futures::JsFuture::from(
                                window.fetch_with_request(&request),
                            )
                            .await
                            {
                                Ok(resp_value) => {
                                    let resp: Response = resp_value.dyn_into().unwrap();

                                    if resp.ok() {
                                        match wasm_bindgen_futures::JsFuture::from(
                                            resp.json().unwrap(),
                                        )
                                        .await
                                        {
                                            Ok(json) => {
                                                // Parse response
                                                if let Ok(challenge_resp) =
                                                    serde_wasm_bindgen::from_value::<
                                                        ChallengeResponse,
                                                    >(
                                                        json
                                                    )
                                                {
                                                    set_challenge_data
                                                        .set(Some(challenge_resp.clone()));
                                                    set_state.update(|s| {
                                                        s.loading = false;
                                                        s.challenge_id =
                                                            Some(challenge_resp.challenge_id);
                                                        s.challenge_type =
                                                            challenge_resp.challenge_type;
                                                    });
                                                } else {
                                                    set_state.update(|s| {
                                                        s.loading = false;
                                                        s.error = Some(
                                                            "Failed to parse challenge response"
                                                                .to_string(),
                                                        );
                                                    });
                                                }
                                            }
                                            Err(_) => {
                                                set_state.update(|s| {
                                                    s.loading = false;
                                                    s.error = Some(
                                                        "Failed to read challenge response"
                                                            .to_string(),
                                                    );
                                                });
                                            }
                                        }
                                    } else {
                                        set_state.update(|s| {
                                            s.loading = false;
                                            s.error = Some(format!("API error: {}", resp.status()));
                                        });
                                    }
                                }
                                Err(_) => {
                                    set_state.update(|s| {
                                        s.loading = false;
                                        s.error = Some(
                                            "Network error: Failed to connect to Authenc"
                                                .to_string(),
                                        );
                                    });
                                }
                            }
                        }
                        Err(_) => {
                            set_state.update(|s| {
                                s.loading = false;
                                s.error = Some("Failed to create request".to_string());
                            });
                        }
                    }
                }
                None => {
                    set_state.update(|s| {
                        s.loading = false;
                        s.error = Some("Window object not available".to_string());
                    });
                }
            }
        });
    });

    // Initialize behavioral data collection if enabled
    if behavioral_analysis {
        Effect::new(move |_| {
            // Announce behavioral analysis is active for accessibility
            if let Some(window) = web_sys::window() {
                let _ = js_sys::Reflect::set(
                    &window,
                    &"sr-announcement".into(),
                    &"Behavioral analysis active for security".into(),
                );
            }
        });
    }

    // Handle challenge refresh
    let refresh_challenge = move |_| {
        set_refresh_trigger.update(|n| *n += 1);
    };

    // Keyboard navigation elements
    let nav_elements = vec![
        "captcha-challenge".to_string(),
        "captcha-input".to_string(),
        "refresh-button".to_string(),
        "audio-button".to_string(),
        "submit-button".to_string(),
    ];

    // Add announcement helper
    let announce = move |message: String| {
        set_announcements.update(|announcements| {
            announcements.push(message);
            // Keep only last 3 announcements
            if announcements.len() > 3 {
                announcements.remove(0);
            }
        });
    };

    view! {
        {if behavioral_analysis {
            view! {
                <super::behavioral_tracker::SimpleBehavioralTracker
                    session_id=session_id.clone()
                    on_data_update=Callback::new(handle_behavioral_update)
                >
                    <CaptchaContainer
                        class=class
                        announcements=announcements
                        current_focus=current_focus
                        set_current_focus=set_current_focus
                        nav_elements=nav_elements
                        state=state
                        set_state=set_state
                        challenge_data=challenge_data
                        accessibility_enabled=accessibility_enabled
                        on_success=on_success
                        on_failure=on_failure
                        refresh_challenge=refresh_challenge
                        show_alternative_inputs=show_alternative_inputs
                        behavioral_analysis=behavioral_analysis
                        behavioral_metrics=behavioral_metrics
                        session_id=session_id
                        validation_status=validation_status
                        set_validation_status=set_validation_status
                        input_value=input_value
                        set_input_value=set_input_value
                    />
                </super::behavioral_tracker::SimpleBehavioralTracker>
            }.into_any()
        } else {
            view! {
                <CaptchaContainer
                    class=class
                    announcements=announcements
                    current_focus=current_focus
                    set_current_focus=set_current_focus
                    nav_elements=nav_elements
                    state=state
                    set_state=set_state
                    challenge_data=challenge_data
                    accessibility_enabled=accessibility_enabled
                    on_success=on_success
                    on_failure=on_failure
                    refresh_challenge=refresh_challenge
                    show_alternative_inputs=show_alternative_inputs
                    behavioral_analysis=behavioral_analysis
                    behavioral_metrics=behavioral_metrics
                    session_id=session_id
                    validation_status=validation_status
                    set_validation_status=set_validation_status
                    input_value=input_value
                    set_input_value=set_input_value
                />
            }.into_any()
        }}
    }
}

/// CAPTCHA container component
#[component]
fn CaptchaContainer(
    class: Option<String>,
    announcements: ReadSignal<Vec<String>>,
    current_focus: ReadSignal<Option<String>>,
    set_current_focus: WriteSignal<Option<String>>,
    nav_elements: Vec<String>,
    state: ReadSignal<CaptchaState>,
    set_state: WriteSignal<CaptchaState>,
    challenge_data: ReadSignal<Option<ChallengeResponse>>,
    accessibility_enabled: bool,
    on_success: Callback<String>,
    on_failure: Option<Callback<String>>,
    refresh_challenge: impl Fn(leptos::ev::MouseEvent) + 'static + Copy + Send,
    show_alternative_inputs: ReadSignal<bool>,
    behavioral_analysis: bool,
    behavioral_metrics: ReadSignal<Option<super::behavioral_tracker::SimpleBehavioralMetrics>>,
    session_id: String,
    validation_status: ReadSignal<ValidationStatus>,
    set_validation_status: WriteSignal<ValidationStatus>,
    input_value: ReadSignal<String>,
    set_input_value: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <div
            class=move || format!(
                "captcha-container bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4 {}",
                class.clone().unwrap_or_default()
            )
            role="region"
            aria-labelledby="captcha-title"
            aria-describedby="captcha-description"
        >
            // Screen reader announcements
            <ScreenReaderAnnouncements announcements=announcements />

            // Keyboard navigation helper
            <KeyboardNavigation
                current_focus=current_focus
                set_focus=set_current_focus
                elements=nav_elements
            />

            <div class="captcha-header mb-4">
                <h3 id="captcha-title" class="text-lg font-semibold text-gray-900 dark:text-gray-100">
                    "Security Verification"
                </h3>
                <p id="captcha-description" class="text-sm text-gray-600 dark:text-gray-400">
                    "Please complete the challenge below to continue. Accessibility options are available."
                </p>
            </div>

            // Difficulty indicator
            <DifficultyIndicator
                current_difficulty=state.get().difficulty
                max_difficulty=10
                attempts=state.get().attempts
                max_attempts=3
            />

            // Validation status indicator
            <ValidationStatusIndicator
                status=validation_status
                on_retry=None
            />

            {move || {
                let current_state = state.get();
                if current_state.loading {
                    view! {
                        <div class="captcha-loading flex items-center justify-center py-8">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-emerald-600"></div>
                            <span class="ml-2 text-gray-600 dark:text-gray-400">"Loading challenge..."</span>
                        </div>
                    }.into_any()
                } else if let Some(error) = current_state.error {
                    view! {
                        <div class="mb-4">
                            <Alert variant=AlertVariant::Error message=error />
                        </div>
                        <Button
                            on_click=Box::new(move || {
                                refresh_challenge(leptos::ev::MouseEvent::new("click").unwrap())
                            })
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Medium
                        >
                            "Try Again"
                        </Button>
                    }.into_any()
                } else {
                    view! {
                        <div class="captcha-challenge space-y-4">
                            {if current_state.challenge_type == ChallengeType::Audio {
                                view! {
                                    <AudioChallenge
                                        _challenge_data=challenge_data.get()
                                        _on_answer=on_success
                                        difficulty=current_state.difficulty
                                    />
                                }.into_any()
                            } else {
                                view! {
                                    <ChallengeDisplay
                                        challenge_type=current_state.challenge_type
                                        difficulty=current_state.difficulty
                                        challenge_data=challenge_data.get()
                                        accessibility_enabled=accessibility_enabled
                                    />
                                }.into_any()
                            }}

                            <ChallengeInput
                                on_submit=on_success
                                on_failure=on_failure
                                accessibility_enabled=accessibility_enabled
                                state=state
                                set_state=set_state
                                refresh_challenge=refresh_challenge
                                validation_status=validation_status
                                set_validation_status=set_validation_status
                                input_value=input_value
                                set_input_value=set_input_value
                            />

                            {if accessibility_enabled && show_alternative_inputs.get() {
                                view! {
                                    <AlternativeInputMethods
                                        on_answer=on_success
                                        challenge_type=state.get().challenge_type
                                    />
                                }.into_any()
                            } else {
                                let _: () = view! {};
                                ().into_any()
                            }}

                            {if behavioral_analysis {
                                view! {
                                    <div class="behavioral-info bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded p-3 mt-2">
                                        <div class="flex items-center justify-between mb-2">
                                            <div class="flex items-center space-x-2 text-xs text-gray-600 dark:text-gray-400">
                                                <span class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></span>
                                                <span>"Behavioral analysis active"</span>
                                            </div>
                                            <div class="text-xs text-gray-500 dark:text-gray-500">
                                                "Session: " {&session_id[..8]} "..."
                                            </div>
                                        </div>
                                        <super::behavioral_tracker::BehavioralMetricsDisplay
                                            metrics=behavioral_metrics
                                            show_details=false
                                        />
                                    </div>
                                }.into_any()
                            } else {
                                let _: () = view! {};
                                ().into_any()
                            }}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
/// Challenge display component
#[component]
pub fn ChallengeDisplay(
    challenge_type: ChallengeType,
    difficulty: u8,
    challenge_data: Option<ChallengeResponse>,
    accessibility_enabled: bool,
) -> impl IntoView {
    let (audio_playing, set_audio_playing) = signal(false);

    // Handle audio challenge playback
    let play_audio = move |_| {
        set_audio_playing.set(true);
        // TODO: Implement actual audio playback
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(2000).await;
            set_audio_playing.set(false);
        });
    };

    view! {
        <div class="challenge-display border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-6 bg-gray-50 dark:bg-gray-900">
            <div class="challenge-header flex items-center justify-between mb-4">
                <div class="challenge-info">
                    <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                        {format!("{:?} Challenge", challenge_type)}
                    </span>
                    <div class="difficulty-indicator flex items-center mt-1">
                        <span class="text-xs text-gray-500 dark:text-gray-400 mr-2">"Difficulty:"</span>
                        <div class="flex space-x-1">
                            {(1..=5).map(|i| {
                                let filled = i <= difficulty.min(5);
                                view! {
                                    <div class=format!(
                                        "w-2 h-2 rounded-full {}",
                                        if filled { "bg-emerald-500" } else { "bg-gray-300 dark:bg-gray-600" }
                                    )></div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                </div>

                {if accessibility_enabled && challenge_type != ChallengeType::Audio {
                    view! {
                        <Button
                            on_click=Box::new(move || {
                                play_audio(leptos::ev::MouseEvent::new("click").unwrap())
                            })
                            variant=ButtonVariant::Ghost
                            size=ButtonSize::Small
                            disabled=audio_playing.get()
                        >
                            {if audio_playing.get() { "🔊 Playing..." } else { "🔊 Audio" }}
                        </Button>
                    }.into_any()
                } else {
                    let _: () = view! {};
                    ().into_any()
                }}
            </div>

            <div class="challenge-content">
                {match challenge_type {
                    ChallengeType::Visual => view! {
                        <div class="visual-challenge">
                            <div class="challenge-question text-lg font-medium text-gray-900 dark:text-gray-100 mb-4">
                                {challenge_data.as_ref().map(|c| c.challenge_data.clone()).unwrap_or_else(|| "Loading...".to_string())}
                            </div>
                            <div class="visual-elements grid grid-cols-2 gap-4">
                                <div class="challenge-image bg-gradient-to-br from-blue-100 to-blue-200 dark:from-blue-900 dark:to-blue-800 rounded-lg h-32 flex items-center justify-center">
                                    <span class="text-2xl font-bold text-blue-800 dark:text-blue-200">"2 + 2"</span>
                                </div>
                                <div class="answer-options space-y-2">
                                    <div class="text-sm text-gray-600 dark:text-gray-400">"Select the correct answer:"</div>
                                    // Visual answer options would go here
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    ChallengeType::Audio => view! {
                        <div class="audio-challenge text-center">
                            <div class="audio-player bg-gradient-to-r from-purple-100 to-pink-100 dark:from-purple-900 dark:to-pink-900 rounded-lg p-6">
                                <div class="text-4xl mb-4">"🎵"</div>
                                <p class="text-gray-700 dark:text-gray-300 mb-4">"Listen to the audio challenge"</p>
                                <Button
                                    on_click=Box::new(move || {
                                        play_audio(leptos::ev::MouseEvent::new("click").unwrap())
                                    })
                                    variant=ButtonVariant::Primary
                                    disabled=audio_playing.get()
                                >
                                    {if audio_playing.get() { "Playing..." } else { "Play Audio" }}
                                </Button>
                            </div>
                        </div>
                    }.into_any(),
                    ChallengeType::Behavioral => view! {
                        <div class="behavioral-challenge">
                            <div class="instruction-text text-center py-8">
                                <div class="text-6xl mb-4">"🖱️"</div>
                                <p class="text-lg text-gray-700 dark:text-gray-300 mb-2">
                                    "Move your mouse naturally in the area below"
                                </p>
                                <p class="text-sm text-gray-500 dark:text-gray-400">
                                    "We analyze your interaction patterns to verify you're human"
                                </p>
                            </div>
                            <div class="interaction-area bg-gradient-to-br from-green-50 to-emerald-50 dark:from-green-900 dark:to-emerald-900 border-2 border-green-200 dark:border-green-700 rounded-lg h-32 cursor-crosshair">
                                // Behavioral tracking area
                            </div>
                        </div>
                    }.into_any(),
                    ChallengeType::Logical => view! {
                        <div class="logical-challenge">
                            <div class="puzzle-container bg-gradient-to-br from-indigo-50 to-purple-50 dark:from-indigo-900 dark:to-purple-900 rounded-lg p-6">
                                <div class="puzzle-question text-lg font-medium text-gray-900 dark:text-gray-100 mb-4">
                                    {challenge_data.as_ref().map(|c| c.challenge_data.clone()).unwrap_or_else(|| "What comes next in the sequence: 2, 4, 6, ?".to_string())}
                                </div>
                                <div class="sequence-display flex items-center justify-center space-x-4 text-2xl font-bold text-indigo-700 dark:text-indigo-300">
                                    <span>"2"</span>
                                    <span>"→"</span>
                                    <span>"4"</span>
                                    <span>"→"</span>
                                    <span>"6"</span>
                                    <span>"→"</span>
                                    <span class="text-gray-400">"?"</span>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    ChallengeType::Hybrid => view! {
                        <div class="hybrid-challenge">
                            <div class="multi-step-challenge space-y-4">
                                <div class="step-indicator flex items-center justify-center space-x-2 mb-4">
                                    <div class="step active bg-emerald-500 text-white rounded-full w-6 h-6 flex items-center justify-center text-xs">"1"</div>
                                    <div class="connector w-8 h-0.5 bg-gray-300"></div>
                                    <div class="step bg-gray-300 text-gray-600 rounded-full w-6 h-6 flex items-center justify-center text-xs">"2"</div>
                                </div>
                                <div class="current-step bg-gradient-to-br from-yellow-50 to-orange-50 dark:from-yellow-900 dark:to-orange-900 rounded-lg p-4">
                                    <p class="text-center text-gray-700 dark:text-gray-300">
                                        "Step 1: " {challenge_data.as_ref().map(|c| c.challenge_data.clone()).unwrap_or_else(|| "Complete the visual challenge first".to_string())}
                                    </p>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                }}
            </div>
        </div>
    }
}

/// Challenge input component
#[component]
pub fn ChallengeInput(
    on_submit: Callback<String>,
    on_failure: Option<Callback<String>>,
    accessibility_enabled: bool,
    state: ReadSignal<CaptchaState>,
    set_state: WriteSignal<CaptchaState>,
    refresh_challenge: impl Fn(leptos::ev::MouseEvent) + 'static + Copy + Send,
    validation_status: ReadSignal<ValidationStatus>,
    set_validation_status: WriteSignal<ValidationStatus>,
    input_value: ReadSignal<String>,
    set_input_value: WriteSignal<String>,
) -> impl IntoView {
    let (is_submitting, set_is_submitting) = signal(false);

    // Handle answer selection for visual challenges
    let handle_answer_click = move |answer: String| {
        if is_submitting.get() {
            return;
        }

        set_is_submitting.set(true);
        set_validation_status.set(ValidationStatus::Validating);

        // For demo purposes, accept "4" as correct answer for "2 + 2"
        let is_correct = answer == "4" || answer == "2 + 2";

        spawn_local(async move {
            // Simulate validation delay
            gloo_timers::future::TimeoutFuture::new(1000).await;

            if is_correct {
                set_validation_status.set(ValidationStatus::Success);
                on_submit.run(format!("captcha_token_{}", js_sys::Date::now() as u64));
            } else {
                set_validation_status.set(ValidationStatus::Failed("Incorrect answer".to_string()));
                set_state.update(|s| {
                    s.attempts += 1;
                    if s.attempts >= 3 {
                        s.error = Some("Too many failed attempts. Please refresh.".to_string());
                    }
                });

                if let Some(failure_callback) = on_failure {
                    failure_callback.run("Incorrect answer".to_string());
                }
            }

            set_is_submitting.set(false);
        });
    };

    // Handle text input for other challenge types
    let handle_input_change = move |ev| {
        let value = event_target_value(&ev);
        set_input_value.set(value);
    };

    let handle_submit = move || {
        let answer = input_value.get().trim().to_string();
        if !answer.is_empty() {
            handle_answer_click(answer);
        }
    };

    view! {
        <div class="challenge-input mt-4">
            {move || {
                let current_state = state.get();
                match current_state.challenge_type {
                    ChallengeType::Visual => view! {
                        <div class="visual-answer-options">
                            <div class="text-sm text-gray-600 dark:text-gray-400 mb-3">
                                "Select the correct answer:"
                            </div>
                            <div class="grid grid-cols-2 gap-3">
                                // Answer option: 4 (correct)
                                <button
                                    class="answer-option bg-blue-100 hover:bg-blue-200 dark:bg-blue-900 dark:hover:bg-blue-800 border-2 border-blue-300 dark:border-blue-700 rounded-lg p-4 text-center transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    disabled=is_submitting.get()
                                    on:click=move |_| handle_answer_click("4".to_string())
                                >
                                    <div class="text-2xl font-bold text-blue-800 dark:text-blue-200">"4"</div>
                                </button>

                                // Answer option: 5 (incorrect)
                                <button
                                    class="answer-option bg-gray-100 hover:bg-gray-200 dark:bg-gray-800 dark:hover:bg-gray-700 border-2 border-gray-300 dark:border-gray-600 rounded-lg p-4 text-center transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-gray-500"
                                    disabled=is_submitting.get()
                                    on:click=move |_| handle_answer_click("5".to_string())
                                >
                                    <div class="text-2xl font-bold text-gray-600 dark:text-gray-400">"5"</div>
                                </button>

                                // Answer option: 3 (incorrect)
                                <button
                                    class="answer-option bg-gray-100 hover:bg-gray-200 dark:bg-gray-800 dark:hover:bg-gray-700 border-2 border-gray-300 dark:border-gray-600 rounded-lg p-4 text-center transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-gray-500"
                                    disabled=is_submitting.get()
                                    on:click=move |_| handle_answer_click("3".to_string())
                                >
                                    <div class="text-2xl font-bold text-gray-600 dark:text-gray-400">"3"</div>
                                </button>

                                // Answer option: 6 (incorrect)
                                <button
                                    class="answer-option bg-gray-100 hover:bg-gray-200 dark:bg-gray-800 dark:hover:bg-gray-700 border-2 border-gray-300 dark:border-gray-600 rounded-lg p-4 text-center transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-gray-500"
                                    disabled=is_submitting.get()
                                    on:click=move |_| handle_answer_click("6".to_string())
                                >
                                    <div class="text-2xl font-bold text-gray-600 dark:text-gray-400">"6"</div>
                                </button>
                            </div>
                        </div>
                    }.into_any(),
                    _ => view! {
                        <div class="text-input-challenge">
                            <div class="flex space-x-2">
                                <input
                                    type="text"
                                    class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-emerald-500 dark:bg-gray-700 dark:text-white"
                                    placeholder="Enter your answer..."
                                    value=input_value
                                    on:input=handle_input_change
                                    disabled=is_submitting.get()
                                />
                                <Button
                                    on_click=Box::new(handle_submit)
                                    variant=ButtonVariant::Primary
                                    disabled=is_submitting.get() || input_value.get().trim().is_empty()
                                >
                                    {if is_submitting.get() { "Verifying..." } else { "Submit" }}
        </Button>
                            </div>
                        </div>
                    }.into_any()
                }
            }}

            // Validation feedback
            {move || {
                match validation_status.get() {
                    ValidationStatus::Success => view! {
                        <div class="mt-2 p-2 bg-green-100 border border-green-300 rounded text-green-700">
                            "✅ Verification successful!"
                        </div>
                    }.into_any(),
                    ValidationStatus::Failed(msg) => view! {
                        <div class="mt-2 p-2 bg-red-100 border border-red-300 rounded text-red-700">
                            "❌ " {msg}
                        </div>
                    }.into_any(),
                    ValidationStatus::Validating => view! {
                        <div class="mt-2 p-2 bg-blue-100 border border-blue-300 rounded text-blue-700">
                            "🔄 Validating..."
                        </div>
                    }.into_any(),
                    _ => {
                        let _: () = view! {};
                        ().into_any()
                    }
                }
            }}

            {if accessibility_enabled {
                view! {
                    <div class="accessibility-options mt-4 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                        <div class="text-sm text-gray-600 dark:text-gray-400 mb-2">
                            "Accessibility Options:"
                        </div>
                        <div class="flex space-x-2">
                            <Button
                                on_click=Box::new(move || {
                                    // Switch to audio challenge
                                    set_state.update(|s| s.challenge_type = ChallengeType::Audio);
                                })
                                variant=ButtonVariant::Ghost
                                size=ButtonSize::Small
                            >
                                "🔊 Audio Challenge"
                            </Button>
                            <Button
                                on_click=Box::new(move || {
                                    refresh_challenge(leptos::ev::MouseEvent::new("click").unwrap())
                                })
                                variant=ButtonVariant::Ghost
                                size=ButtonSize::Small
                            >
                                "🔄 New Challenge"
                            </Button>
                        </div>
                    </div>
                }.into_any()
            } else {
                let _: () = view! {};
                ().into_any()
            }}
        </div>
    }
}
