//! CAPTCHA Component Implementation
//!
//! Main CAPTCHA component using Leptos and shared component library

use super::accessibility::*;
use super::challenge_parser;
use super::types::*;
use super::validation_feedback::*;
use crate::components::feedback::Alert;
use crate::components::forms::Button;
use crate::core::types::{AlertVariant, ButtonSize, ButtonVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Main CAPTCHA component
#[component]
#[allow(unused_variables)]
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
    /// External reset trigger: bump this counter (e.g. after a failed login that
    /// consumed the solved challenge, #49) to fetch a FRESH challenge and clear
    /// the previous answer. Without it the widget would keep showing an
    /// already-consumed/expired challenge.
    #[prop(optional, into)]
    reset: Option<Signal<u32>>,
) -> impl IntoView {
    let (state, set_state) = signal(CaptchaState::default());
    let (challenge_data, set_challenge_data) = signal(None::<ChallengeResponse>);

    // Accessibility state
    let (current_focus, set_current_focus) = signal(None::<String>);
    let (announcements, set_announcements) = signal(Vec::<String>::new());
    let (show_alternative_inputs, _set_show_alternative_inputs) = signal(false);

    // Behavioral analysis state (New Implementation)
    let (behavioral_data, _set_behavioral_data) = super::behavioral::use_behavioral_collector();

    let session_id = format!("captcha_session_{}", js_sys::Date::now() as u64);

    // Validation feedback state
    let (validation_status, set_validation_status) =
        signal(super::validation_feedback::ValidationStatus::Idle);
    let (input_value, set_input_value) = signal(String::new());

    // trigger used to force refresh of a new challenge whenever
    // the counter increments (e.g. after wrong answers)
    let (refresh_trigger, set_refresh_trigger) = signal(0);

    // Initialize with provided difficulty
    if let Some(diff) = difficulty {
        set_state.update(|s| s.difficulty = diff);
    }

    // Clone session_id for use in async closures
    let session_id_clone = session_id.clone();
    let session_id_verify = session_id.clone();

    // Verification logic (lifted to parent to support AlternativeInputs)
    let handle_verification = move |answer: String| {
        set_validation_status.set(ValidationStatus::Validating);

        let behavioral_data_val = behavioral_data.get();
        // Use state for challenge_id
        let challenge_id_val = state.get().challenge_id.unwrap_or_default();
        let session_id_val = session_id_verify.clone();
        let set_refresh = set_refresh_trigger;

        // Clone callbacks for async block
        let on_success_clone = on_success;
        let on_failure_clone = on_failure;

        spawn_local(async move {
            // Get backend URL - use window.location.origin for same-origin requests
            // Routes through backend portal service which calls Authenc via gRPC
            let backend_url = web_sys::window()
                .and_then(|w| w.location().origin().ok())
                .unwrap_or_else(|| "http://localhost:8080".to_string());

            let verify_url = format!("{}/api/captcha/verify", backend_url);

            // Prepare validation request with session_id
            let validation_request = ValidationRequest {
                challenge_id: challenge_id_val,
                answer,
                session_id: Some(session_id_val.clone()),
                behavioral_data: Some(behavioral_data_val),
            };

            let mut success = false;
            let mut error_msg = "Verification failed".to_string();
            let mut val_resp: Option<ValidationResponse> = None;

            // Make API call to Authenc
            match web_sys::window() {
                Some(window) => {
                    use wasm_bindgen::{JsCast, JsValue};
                    use web_sys::{Request, RequestInit, RequestMode, Response};

                    let opts = RequestInit::new();
                    opts.set_method("POST");
                    opts.set_mode(RequestMode::Cors);

                    // Set body
                    if let Ok(body_str) = serde_json::to_string(&validation_request) {
                        opts.set_body(&JsValue::from_str(&body_str));
                    }

                    match Request::new_with_str_and_init(&verify_url, &opts) {
                        Ok(request) => {
                            let _ = request.headers().set("Content-Type", "application/json");

                            match wasm_bindgen_futures::JsFuture::from(
                                window.fetch_with_request(&request),
                            )
                            .await
                            {
                                Ok(resp_value) => {
                                    let resp: Response = resp_value.dyn_into().unwrap();
                                    if resp.ok() {
                                        if let Ok(json) = wasm_bindgen_futures::JsFuture::from(
                                            resp.json().unwrap(),
                                        )
                                        .await
                                            && let Ok(parsed_resp) = serde_wasm_bindgen::from_value::<
                                                ValidationResponse,
                                            >(
                                                json
                                            )
                                        {
                                            if parsed_resp.success {
                                                success = true;
                                            } else {
                                                error_msg = parsed_resp.message.clone();
                                            }
                                            val_resp = Some(parsed_resp);
                                        }
                                    } else {
                                        error_msg = format!("API error: {}", resp.status());
                                    }
                                }
                                Err(_) => {
                                    error_msg = "Network error".to_string();
                                }
                            }
                        }
                        Err(_) => {
                            error_msg = "Request creation failed".to_string();
                        }
                    }
                }
                None => {
                    error_msg = "Window not available".to_string();
                }
            }

            if success {
                set_validation_status.set(ValidationStatus::Success);
                let token = val_resp
                    .and_then(|r| r.token)
                    .unwrap_or_else(|| format!("captcha_verified_{}", session_id_val));
                on_success_clone.run(token);
            } else {
                set_validation_status.set(ValidationStatus::Failed(error_msg.clone()));
                set_state.update(|s| {
                    s.attempts += 1;
                    if s.attempts >= 3 {
                        s.error =
                            Some("Terlalu banyak percobaan gagal. Silakan muat ulang.".to_string());
                    }
                });

                // force a refresh right away so a new set of characters is shown
                set_refresh.update(|n| *n += 1);

                if let Some(failure_callback) = on_failure_clone {
                    failure_callback.run(error_msg);
                }
            }
        });
    };

    Effect::new(move |_| {
        // Track refresh trigger (internal: wrong-answer refresh) and the optional
        // external reset signal (parent-driven, e.g. after a failed login that
        // consumed the solved challenge, #49).
        let _ = refresh_trigger.get();
        if let Some(reset) = reset {
            let _ = reset.get();
        }

        // A fresh challenge invalidates any previously typed answer / feedback.
        set_input_value.set(String::new());
        set_validation_status.set(super::validation_feedback::ValidationStatus::Idle);

        set_state.update(|s| {
            s.loading = true;
            s.error = None;
        });

        let session_id_for_spawn = session_id_clone.clone();

        spawn_local(async move {
            // Get backend URL - use window.location.origin for same-origin requests
            // Routes through backend portal service which calls Authenc via gRPC
            let backend_url = web_sys::window()
                .and_then(|w| w.location().origin().ok())
                .unwrap_or_else(|| "http://localhost:8080".to_string());

            let challenge_url = format!("{}/api/captcha/challenge", backend_url);

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

    // Keyboard navigation elements (audio button removed)
    let nav_elements = vec![
        "captcha-challenge".to_string(),
        "captcha-input".to_string(),
        "refresh-button".to_string(),
        "submit-button".to_string(),
    ];

    // Add announcement helper
    let _announce = move |message: String| {
        set_announcements.update(|announcements| {
            announcements.push(message);
            // Keep only last 3 announcements
            if announcements.len() > 3 {
                announcements.remove(0);
            }
        });
    };

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
            behavioral_data=behavioral_data
            session_id=session_id
            validation_status=validation_status
            set_validation_status=set_validation_status
            input_value=input_value
            set_input_value=set_input_value
            on_verify_answer=Callback::new(handle_verification)
        />
    }
}

/// CAPTCHA container component
#[component]
#[allow(unused_variables)]
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
    on_verify_answer: Callback<String>,
    refresh_challenge: impl Fn(leptos::ev::MouseEvent) + 'static + Copy + Send,
    show_alternative_inputs: ReadSignal<bool>,
    behavioral_analysis: bool,
    behavioral_data: ReadSignal<BehavioralData>,
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
                <h3 id="captcha-title" class="text-lg font-bold text-gray-900 dark:text-white">
                    "Verifikasi Keamanan"
                </h3>
                <p id="captcha-description" class="text-sm font-medium text-gray-600 dark:text-gray-300">
                    "Selesaikan tantangan di bawah ini"
                </p>
            </div>


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
                            <span class="ml-2 text-gray-600 dark:text-gray-400">"Memuat tantangan..."</span>
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
                            "Coba Lagi"
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
                                        _difficulty=current_state.difficulty
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
                                behavioral_data=behavioral_data
                                session_id=session_id.clone()
                                challenge_id=current_state.challenge_id.clone()
                                challenge_data=challenge_data.get()
                            />

                            {if accessibility_enabled && show_alternative_inputs.get() {
                                view! {
                                    <AlternativeInputs
                                        on_answer=on_verify_answer
                                        challenge_type=state.get().challenge_type
                                    />
                                }.into_any()
                            } else {

                                ().into_any()
                            }}

                            // Behavioral analysis runs silently (no debug UI shown)
                            { ().into_any()}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
/// Challenge display component
#[component]
#[allow(unused_variables)]
pub fn ChallengeDisplay(
    challenge_type: ChallengeType,
    _difficulty: u8,
    challenge_data: Option<ChallengeResponse>,
    accessibility_enabled: bool,
) -> impl IntoView {
    let (audio_playing, set_audio_playing) = signal(false);

    // Handle audio challenge playback
    let challenge_data_for_audio = challenge_data.clone();
    let play_audio = move |_| {
        set_audio_playing.set(true);
        let challenge_data = challenge_data_for_audio.clone();

        spawn_local(async move {
            let mut audio_played = false;

            if let Some(data) = challenge_data
                && play_audio_content(data.challenge_data).await.is_ok()
            {
                audio_played = true;
            }

            if !audio_played {
                // Fallback if no audio played
                gloo_timers::future::TimeoutFuture::new(1000).await;
            }

            set_audio_playing.set(false);
        });
    };

    view! {
        <div class="challenge-display border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-6 bg-gray-50 dark:bg-gray-900">
            <div class="challenge-header flex items-center justify-between mb-4">
            </div>

            <div class="challenge-content">
                {move || {
                    let parsed = challenge_parser::parse_challenge(challenge_data.clone());
                    let play_audio_fn = play_audio.clone();

                    match challenge_type {
                    ChallengeType::Visual => match parsed.challenge_type.as_str() {
                            "text_recognition" => view! {
                                <div class="text-recognition-challenge">
                                    <div class="challenge-question text-lg font-medium text-gray-900 dark:text-gray-100 mb-4">
                                        {parsed.instructions.clone()}
                                    </div>
                                    <div class="text-display rounded-lg p-2 flex items-center justify-center select-none"
                                        style="user-select: none; -webkit-user-select: none; pointer-events: none;">
                                        {move || {
                                            if let Some(ref svg_markup) = parsed.svg {
                                                // Render server-generated SVG with noise/distortion
                                                view! {
                                                    <div
                                                        class="captcha-svg-container"
                                                        style="user-select: none; -webkit-user-select: none; pointer-events: none; width: 100%; max-width: 100%; overflow: hidden;"
                                                        inner_html=svg_markup.clone()
                                                    ></div>
                                                }.into_any()
                                            } else {
                                                // Fallback: render individual characters with CSS distortion
                                                let text = parsed.text_to_recognize.clone().unwrap_or_default();
                                                let chars: Vec<char> = text.chars().collect();
                                                view! {
                                                    <div class="flex space-x-1 select-none" style="letter-spacing: 0.5em; text-shadow: 2px 2px 4px rgba(0,0,0,0.1);">
                                                        {chars.into_iter().map(|ch| {
                                                            let rot = js_sys::Math::random() * 30.0 - 15.0;
                                                            let style_str = format!("transform: rotate({rot:.1}deg); display: inline-block;");
                                                            view! {
                                                                <span class="text-4xl font-mono font-bold text-blue-800 dark:text-blue-200 inline-block"
                                                                    style=style_str>
                                                                    {ch.to_string()}
                                                                </span>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                }.into_any()
                                            }
                                        }}
                                    </div>
                                </div>
                            }.into_any(),
                            "image_selection" => view! {
                                <div class="image-selection-challenge">
                                    <div class="challenge-question text-lg font-medium text-gray-900 dark:text-gray-100 mb-4">
                                        {parsed.instructions.clone()}
                                    </div>
                                    <div class="image-grid grid gap-2"
                                        style={format!("grid-template-columns: repeat({}, 1fr);", parsed.grid_size)}>
                                        {parsed.images.iter().map(|img| {
                                            let idx = img.index;
                                            view! {
                                                <div class="image-cell aspect-square bg-gradient-to-br from-gray-100 to-gray-200 dark:from-gray-700 dark:to-gray-800 rounded cursor-pointer hover:ring-2 hover:ring-blue-500 transition-all"
                                                    data-index={idx.to_string()}>
                                                    <div class="w-full h-full flex items-center justify-center text-gray-500">
                                                        {format!("📷 {}", idx + 1)}
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                    <p class="text-sm text-gray-500 mt-2">"Click images to select/deselect"</p>
                                </div>
                            }.into_any(),
                            _ => view! {
                                <div class="visual-challenge">
                                    <div class="challenge-question text-lg font-medium text-gray-900 dark:text-gray-100 mb-4">
                                        {parsed.instructions.clone()}
                                    </div>
                                    <div class="visual-elements grid grid-cols-2 gap-4">
                                        <div class="challenge-image bg-gradient-to-br from-blue-100 to-blue-200 dark:from-blue-900 dark:to-blue-800 rounded-lg h-32 flex items-center justify-center">
                                            <span class="text-2xl font-bold text-blue-800 dark:text-blue-200">
                                                {parsed.display_data.clone()}
                                            </span>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        },
                    ChallengeType::Audio => view! {
                        <div class="audio-challenge text-center">
                            <div class="audio-player bg-gradient-to-r from-purple-100 to-pink-100 dark:from-purple-900 dark:to-pink-900 rounded-lg p-6">
                                <div class="text-4xl mb-4">"🎵"</div>
                                <p class="text-gray-700 dark:text-gray-300 mb-4">"Listen to the audio challenge"</p>
                                <Button
                                    on_click=Box::new(move || {
                                        play_audio_fn(leptos::ev::MouseEvent::new("click").unwrap())
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
                    ChallengeType::Logical => {
                        let logical_parsed = challenge_parser::parse_challenge(challenge_data.clone());
                        view! {
                        <div class="logical-challenge">
                            <div class="puzzle-container bg-gradient-to-br from-indigo-50 to-purple-50 dark:from-indigo-900 dark:to-purple-900 rounded-lg p-6">
                                <div class="puzzle-question text-lg font-medium text-gray-900 dark:text-gray-100 mb-4">
                                    {logical_parsed.instructions.clone()}
                                </div>
                                <div class="sequence-display flex items-center justify-center space-x-4 text-2xl font-bold text-indigo-700 dark:text-indigo-300">
                                    <span>{logical_parsed.display_data.clone()}</span>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                    },
                    ChallengeType::Hybrid => {
                        let hybrid_parsed = challenge_parser::parse_challenge(challenge_data.clone());
                        view! {
                        <div class="hybrid-challenge">
                            <div class="multi-step-challenge space-y-4">
                                <div class="step-indicator flex items-center justify-center space-x-2 mb-4">
                                    <div class="step active bg-emerald-500 text-white rounded-full w-6 h-6 flex items-center justify-center text-xs">"1"</div>
                                    <div class="connector w-8 h-0.5 bg-gray-300"></div>
                                    <div class="step bg-gray-300 text-gray-600 rounded-full w-6 h-6 flex items-center justify-center text-xs">"2"</div>
                                </div>
                                <div class="current-step bg-gradient-to-br from-yellow-50 to-orange-50 dark:from-yellow-900 dark:to-orange-900 rounded-lg p-4">
                                    <p class="text-center text-gray-700 dark:text-gray-300">
                                        "Step 1: " {hybrid_parsed.instructions.clone()}
                                    </p>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                    },
                }}}
            </div>
        </div>
    }
}

/// Challenge input component
#[component]
#[allow(unused_variables)]
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
    behavioral_data: ReadSignal<BehavioralData>,
    session_id: String,
    challenge_id: Option<String>,
    challenge_data: Option<ChallengeResponse>,
) -> impl IntoView {
    let (is_submitting, set_is_submitting) = signal(false);
    let session_id_store = StoredValue::new(session_id);
    let challenge_id_store = StoredValue::new(challenge_id);

    // Handle answer selection for visual challenges
    let handle_answer_click = move |answer: String| {
        if is_submitting.get() {
            return;
        }

        set_is_submitting.set(true);
        set_validation_status.set(ValidationStatus::Validating);

        let behavioral_data_val = behavioral_data.get();
        let challenge_id_val = challenge_id_store.get_value().unwrap_or_default();
        let session_id_val = session_id_store.get_value();

        spawn_local(async move {
            // Get backend URL - use window.location.origin
            let backend_url = web_sys::window()
                .and_then(|w| w.location().origin().ok())
                .unwrap_or_else(|| "http://localhost:8080".to_string());

            let verify_url = format!("{}/api/captcha/verify", backend_url);

            // Prepare validation request with session_id
            let validation_request = ValidationRequest {
                challenge_id: challenge_id_val,
                answer,
                session_id: Some(session_id_val.clone()),
                behavioral_data: Some(behavioral_data_val),
            };

            let mut success = false;
            let mut error_msg = "Verification failed".to_string();
            let mut val_resp: Option<ValidationResponse> = None;

            // Make API call to Authenc
            match web_sys::window() {
                Some(window) => {
                    use wasm_bindgen::{JsCast, JsValue};
                    use web_sys::{Request, RequestInit, RequestMode, Response};

                    let opts = RequestInit::new();
                    opts.set_method("POST");
                    opts.set_mode(RequestMode::Cors);

                    // Set body
                    if let Ok(body_str) = serde_json::to_string(&validation_request) {
                        opts.set_body(&JsValue::from_str(&body_str));
                    }

                    match Request::new_with_str_and_init(&verify_url, &opts) {
                        Ok(request) => {
                            let _ = request.headers().set("Content-Type", "application/json");

                            match wasm_bindgen_futures::JsFuture::from(
                                window.fetch_with_request(&request),
                            )
                            .await
                            {
                                Ok(resp_value) => {
                                    let resp: Response = resp_value.dyn_into().unwrap();
                                    if resp.ok() {
                                        if let Ok(json) = wasm_bindgen_futures::JsFuture::from(
                                            resp.json().unwrap(),
                                        )
                                        .await
                                            && let Ok(parsed_resp) = serde_wasm_bindgen::from_value::<
                                                ValidationResponse,
                                            >(
                                                json
                                            )
                                        {
                                            if parsed_resp.success {
                                                success = true;
                                            } else {
                                                error_msg = parsed_resp.message.clone();
                                            }
                                            val_resp = Some(parsed_resp);
                                        }
                                    } else {
                                        error_msg = format!("API error: {}", resp.status());
                                    }
                                }
                                Err(_) => {
                                    error_msg = "Network error".to_string();
                                }
                            }
                        }
                        Err(_) => {
                            error_msg = "Request creation failed".to_string();
                        }
                    }
                }
                None => {
                    error_msg = "Window not available".to_string();
                }
            }

            if success {
                set_validation_status.set(ValidationStatus::Success);
                let token = val_resp
                    .and_then(|r| r.token)
                    .unwrap_or_else(|| format!("captcha_verified_{}", session_id_val));
                on_submit.run(token);
            } else {
                set_validation_status.set(ValidationStatus::Failed(error_msg.clone()));
                // Immediately refresh the challenge to prevent brute-force
                refresh_challenge(leptos::ev::MouseEvent::new("click").unwrap());
                let mut should_notify_parent = false;
                set_state.update(|s| {
                    s.attempts += 1;
                    if s.attempts >= 3 {
                        s.error =
                            Some("Terlalu banyak percobaan gagal. Silakan muat ulang.".to_string());
                        should_notify_parent = true;
                    }
                });

                // Only notify parent on critical failure (exhausted retries)
                // ValidationStatusIndicator handles normal failures
                if should_notify_parent && let Some(failure_callback) = on_failure {
                    failure_callback
                        .run("Terlalu banyak percobaan gagal. Silakan muat ulang.".to_string());
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
                let parsed = challenge_parser::parse_challenge(challenge_data.clone());

                match current_state.challenge_type {
                    ChallengeType::Visual => {
                        match parsed.challenge_type.as_str() {
                            "text_recognition" => view! {
                                <div class="text-input-challenge">
                                    <div class="text-sm text-gray-600 dark:text-gray-400 mb-3">
                                        "Ketik karakter yang Anda lihat di atas (perhatikan huruf besar/kecil):"
                                    </div>
                                    <div class="flex space-x-2">
                                        <input
                                            type="text"
                                            class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-emerald-500 dark:bg-gray-700 dark:text-white font-mono text-lg tracking-widest"
                                            placeholder="Masukkan teks..."
                                            value=input_value
                                            on:input=handle_input_change
                                            disabled=is_submitting.get()
                                            maxlength="10"
                                            autocomplete="off"
                                            spellcheck="false"
                                        />
                                        <Button
                                            on_click=Box::new(handle_submit)
                                            variant=ButtonVariant::Primary
                                            disabled=is_submitting.get() || input_value.get().trim().is_empty()
                                        >
                                            {if is_submitting.get() { "Memverifikasi..." } else { "Kirim" }}
                                        </Button>
                                    </div>
                                </div>
                            }.into_any(),
                            "image_selection" => view! {
                                <div class="image-selection-input">
                                    <div class="text-sm text-gray-600 dark:text-gray-400 mb-3">
                                        "Gambar terpilih akan disorot. Klik Kirim jika sudah selesai."
                                    </div>
                                    <div class="flex space-x-2">
                                        <input
                                            type="text"
                                            class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-emerald-500 dark:bg-gray-700 dark:text-white"
                                            placeholder="Masukkan nomor gambar (cth: 1,3,5)..."
                                            value=input_value
                                            on:input=handle_input_change
                                            disabled=is_submitting.get()
                                        />
                                        <Button
                                            on_click=Box::new(handle_submit)
                                            variant=ButtonVariant::Primary
                                            disabled=is_submitting.get() || input_value.get().trim().is_empty()
                                        >
                                            {if is_submitting.get() { "Memverifikasi" } else { "Kirim" }}
                                        </Button>
                                    </div>
                                </div>
                            }.into_any(),
                            _ => {
                                let options = parsed.options.clone();
                                if options.is_empty() || options[0] == "..." {
                                    // No options, show text input
                                    view! {
                                        <div class="text-input-challenge">
                                            <div class="flex space-x-2">
                                                <input
                                                    type="text"
                                                    class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-emerald-500 dark:bg-gray-700 dark:text-white"
                                                    placeholder="Masukkan jawaban Anda..."
                                                    value=input_value
                                                    on:input=handle_input_change
                                                    disabled=is_submitting.get()
                                                />
                                                <Button
                                                    on_click=Box::new(handle_submit)
                                                    variant=ButtonVariant::Primary
                                                    disabled=is_submitting.get() || input_value.get().trim().is_empty()
                                                >
                                                    {if is_submitting.get() { "Memverifikasi..." } else { "Kirim" }}
                                                </Button>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    // Show option buttons
                                    view! {
                                        <div class="visual-answer-options">
                                            <div class="text-sm text-gray-600 dark:text-gray-400 mb-3">
                                                "Pilih jawaban yang benar:"
                                            </div>
                                            <div class="grid grid-cols-2 gap-3">
                                                {options.into_iter().map(|option| {
                                                    let option_clone = option.clone();
                                                    view! {
                                                        <button
                                                            class="answer-option bg-blue-100 hover:bg-blue-200 dark:bg-blue-900 dark:hover:bg-blue-800 border-2 border-blue-300 dark:border-blue-700 rounded-lg p-4 text-center transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                                                            disabled=is_submitting.get()
                                                            on:click=move |_| handle_answer_click(option_clone.clone())
                                                        >
                                                            <div class="text-2xl font-bold text-blue-800 dark:text-blue-200">{option}</div>
                                                        </button>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    }.into_any()
                                }
                            }
                        }
                    },
                    _ => view! {
                        <div class="text-input-challenge">
                            <div class="flex space-x-2">
                                <input
                                    type="text"
                                    class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-emerald-500 dark:bg-gray-700 dark:text-white"
                                    placeholder="Masukkan jawaban Anda..."
                                    value=input_value
                                    on:input=handle_input_change
                                    disabled=is_submitting.get()
                                />
                                <Button
                                    on_click=Box::new(handle_submit)
                                    variant=ButtonVariant::Primary
                                    disabled=is_submitting.get() || input_value.get().trim().is_empty()
                                >
                                    {if is_submitting.get() { "Memverifikasi..." } else { "Kirim" }}
                                </Button>
                            </div>
                        </div>
                    }.into_any()
                }
            }}

            // Validation feedback is shown by ValidationStatusIndicator in CaptchaContainer

            { /* accessibility options removed per updated requirements */

                ().into_any()
            }
        </div>
    }
}
