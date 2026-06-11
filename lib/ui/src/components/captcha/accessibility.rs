//! Accessibility Features for CAPTCHA Component
//!
//! Implements WCAG 2.1 AA compliance for CAPTCHA challenges

use super::types::*;
use crate::components::forms::Button;
use crate::core::types::{ButtonSize, ButtonVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;

/// Audio challenge component for visually impaired users
#[component]
pub fn AudioChallenge(
    _challenge_data: Option<ChallengeResponse>,
    _on_answer: Callback<String>,
    difficulty: u8,
) -> impl IntoView {
    let (audio_playing, set_audio_playing) = signal(false);
    let (audio_text, set_audio_text) = signal(String::new());
    let (playback_speed, set_playback_speed) = signal(1.0);

    // Generate audio challenge text based on difficulty
    let generate_audio_text = move || {
        match difficulty {
            1..=3 => "Please type the number: four",
            4..=6 => "Please type the result of: two plus two",
            7..=8 => "Please type the word that rhymes with 'door': four",
            _ => "Please solve: What is two multiplied by two?",
        }
        .to_string()
    };

    // Initialize audio text
    Effect::new(move |_| {
        set_audio_text.set(generate_audio_text());
    });

    // Play audio using Web Speech API
    let play_audio = move |_| {
        set_audio_playing.set(true);

        spawn_local(async move {
            if let Some(window) = web_sys::window()
                && let Ok(speech_synthesis) =
                    js_sys::Reflect::get(&window, &"speechSynthesis".into())
                && !speech_synthesis.is_undefined()
            {
                // Create speech utterance
                let utterance =
                    web_sys::SpeechSynthesisUtterance::new_with_text(&audio_text.get()).unwrap();
                utterance.set_rate(playback_speed.get() as f32);
                utterance.set_volume(0.8);

                // Set up event handlers
                let set_audio_playing_clone = set_audio_playing;
                let onend = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                    set_audio_playing_clone.set(false);
                })
                    as Box<dyn Fn()>);

                utterance.set_onend(Some(onend.as_ref().unchecked_ref()));
                onend.forget(); // Keep closure alive

                // Speak the text
                let synthesis: web_sys::SpeechSynthesis = speech_synthesis.into();
                synthesis.speak(&utterance);
            }

            // Fallback timeout
            gloo_timers::future::TimeoutFuture::new(5000).await;
            set_audio_playing.set(false);
        });
    };

    // Stop audio playback
    let stop_audio = move |_| {
        if let Some(window) = web_sys::window()
            && let Ok(speech_synthesis) = js_sys::Reflect::get(&window, &"speechSynthesis".into())
            && !speech_synthesis.is_undefined()
        {
            let synthesis: web_sys::SpeechSynthesis = speech_synthesis.into();
            synthesis.cancel();
        }
        set_audio_playing.set(false);
    };

    // Adjust playback speed
    let adjust_speed = move |speed: f64| {
        set_playback_speed.set(speed);
    };

    view! {
         <div class="audio-challenge bg-blue-50 dark:bg-blue-900 border-2 border-blue-200 dark:border-blue-700 rounded-lg p-6">
             <div class="audio-header mb-4">
                 <h3 class="text-lg font-semibold text-blue-900 dark:text-blue-100 mb-2">
                     "🔊 Audio Challenge"
                 </h3>
                 <p class="text-sm text-blue-700 dark:text-blue-300">
                     "Dengarkan dengan seksama dan ketik jawaban Anda di bawah"
                 </p>
             </div>

             <div class="audio-controls space-y-4">
                 <div class="playback-controls flex items-center justify-center space-x-4">
                     <Button
                         on_click=Box::new(move || {
                             play_audio(leptos::ev::MouseEvent::new("click").unwrap())
                         })
                         variant=ButtonVariant::Primary
                         size=ButtonSize::Large
                         disabled=audio_playing.get()
                     >
                         {if audio_playing.get() { "🔊 Memutar..." } else { "▶️ Putar Audio" }}
                     </Button>

                     {if audio_playing.get() {
                         view! {
                             <Button
                                 on_click=Box::new(move || {
                                     stop_audio(leptos::ev::MouseEvent::new("click").unwrap())
                                 })
                                 variant=ButtonVariant::Secondary
                                 size=ButtonSize::Medium
                             >
                                 "⏹️ Stop"
                             </Button>
                         }.into_any()
                     } else {

                         ().into_any()
                     }}
                 </div>

                 <div class="speed-controls">
                     <label class="block text-sm font-medium text-blue-700 dark:text-blue-300 mb-2">
                         "Kecepatan Putar"
                     </label>
                     <div class="flex items-center space-x-2">
                         <Button
                             on_click=Box::new(move || adjust_speed(0.75))
                             variant=if playback_speed.get() == 0.75 { ButtonVariant::Primary } else { ButtonVariant::Ghost }
                             size=ButtonSize::Small
                         >
                             "0.75x"
                         </Button>
                         <Button
                             on_click=Box::new(move || adjust_speed(1.0))
    variant=if playback_speed.get() == 1.0 { ButtonVariant::Primary } else { ButtonVariant::Ghost }
                             size=ButtonSize::Small
                         >
                             "1x"
                         </Button>
                         <Button
                             on_click=Box::new(move || adjust_speed(1.25))
                             variant=if playback_speed.get() == 1.25 { ButtonVariant::Primary } else { ButtonVariant::Ghost }
                             size=ButtonSize::Small
                         >
                             "1.25x"
                         </Button>
                     </div>
                 </div>

                 <div class="text-alternative bg-white dark:bg-gray-800 border border-blue-200 dark:border-blue-600 rounded p-3">
                     <p class="text-sm text-gray-700 dark:text-gray-300">
                         <strong>"Versi teks:"</strong> " " {audio_text.get()}
                     </p>
                 </div>
             </div>
         </div>
     }
}

/// Keyboard navigation helper component
#[component]
pub fn KeyboardNavigation(
    current_focus: ReadSignal<Option<String>>,
    set_focus: WriteSignal<Option<String>>,
    elements: Vec<String>,
) -> impl IntoView {
    // Handle keyboard navigation
    let handle_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let current = current_focus.get();
        let current_index = current
            .as_ref()
            .and_then(|id| elements.iter().position(|e| e == id))
            .unwrap_or(0);

        match ev.key().as_str() {
            "Tab" => {
                if ev.shift_key() {
                    // Previous element
                    let prev_index = if current_index == 0 {
                        elements.len() - 1
                    } else {
                        current_index - 1
                    };
                    set_focus.set(Some(elements[prev_index].clone()));
                } else {
                    // Next element
                    let next_index = (current_index + 1) % elements.len();
                    set_focus.set(Some(elements[next_index].clone()));
                }
                ev.prevent_default();
            }
            "Enter" | " " => {
                // Activate focused element
                if let Some(focused_id) = current_focus.get() {
                    // Trigger click event on focused element
                    if let Some(window) = web_sys::window()
                        && let Some(document) = window.document()
                        && let Some(element) = document.get_element_by_id(&focused_id)
                    {
                        let _ = element.dispatch_event(&web_sys::Event::new("click").unwrap());
                    }
                }
                ev.prevent_default();
            }
            "Escape" => {
                set_focus.set(None);
            }
            _ => {}
        }
    };

    view! {
        <div
            class="sr-only"
            tabindex="0"
            on:keydown=handle_keydown
            aria-label="Keyboard navigation helper"
        >
            "Use Tab to navigate, Enter or Space to activate, Escape to clear focus"
        </div>
    }
}

/// Screen reader announcements component
#[component]
pub fn ScreenReaderAnnouncements(announcements: ReadSignal<Vec<String>>) -> impl IntoView {
    view! {
        <div
            aria-live="polite"
            aria-atomic="true"
            class="sr-only"
        >
            {move || {
                announcements.get().into_iter().map(|announcement| {
                    view! { <div>{announcement}</div> }
                }).collect::<Vec<_>>()
            }}
        </div>
    }
}

/// Alternative input methods component for accessibility
#[component]
pub fn AlternativeInputs(
    challenge_type: ChallengeType,
    on_answer: Callback<String>,
) -> impl IntoView {
    let (selected_option, set_selected_option) = signal(None::<String>);
    let (voice_input_active, set_voice_input_active) = signal(false);

    // Store recognition instance for cleanup
    let recognition_ref = StoredValue::new(None::<SendWrapper<js_sys::Object>>);

    // Cleanup on component unmount
    on_cleanup(move || {
        if let Some(wrapped_recognition) = recognition_ref.get_value() {
            let recognition = wrapped_recognition.take();
            if let Ok(abort_fn) = js_sys::Reflect::get(&recognition, &"abort".into()) {
                let _ = js_sys::Reflect::apply(
                    abort_fn.unchecked_ref::<js_sys::Function>(),
                    &recognition,
                    &js_sys::Array::new(),
                );
            }
        }
    });

    // Voice input handler
    let start_voice_input = move |_| {
        set_voice_input_active.set(true);

        spawn_local(async move {
            if let Some(window) = web_sys::window() {
                // Check for speech recognition support
                let speech_recognition =
                    js_sys::Reflect::get(&window, &"webkitSpeechRecognition".into())
                        .or_else(|_| js_sys::Reflect::get(&window, &"SpeechRecognition".into()));

                if let Ok(recognition_constructor) = speech_recognition
                    && !recognition_constructor.is_undefined()
                {
                    // Create speech recognition instance
                    if let Ok(recognition) = js_sys::Reflect::construct(
                        recognition_constructor.unchecked_ref::<js_sys::Function>(),
                        &js_sys::Array::new(),
                    ) {
                        // Store reference for cleanup
                        if let Ok(obj) = recognition.clone().dyn_into::<js_sys::Object>() {
                            recognition_ref.set_value(Some(SendWrapper::new(obj)));
                        }

                        // Configure recognition
                        let _ =
                            js_sys::Reflect::set(&recognition, &"continuous".into(), &false.into());
                        let _ = js_sys::Reflect::set(
                            &recognition,
                            &"interimResults".into(),
                            &false.into(),
                        );
                        let _ = js_sys::Reflect::set(&recognition, &"lang".into(), &"en-US".into());

                        // Set up result handler
                        let on_answer_clone = on_answer;
                        let set_voice_input_active_clone = set_voice_input_active;
                        let onresult = wasm_bindgen::closure::Closure::wrap(Box::new(
                            move |event: web_sys::Event| {
                                // Extract speech result
                                if let Ok(results) = js_sys::Reflect::get(&event, &"results".into())
                                    && let Ok(result) = js_sys::Reflect::get(&results, &0.into())
                                    && let Ok(alternative) =
                                        js_sys::Reflect::get(&result, &0.into())
                                    && let Ok(transcript) =
                                        js_sys::Reflect::get(&alternative, &"transcript".into())
                                    && let Some(text) = transcript.as_string()
                                {
                                    on_answer_clone.run(text.trim().to_string());
                                }
                                set_voice_input_active_clone.set(false);
                            },
                        )
                            as Box<dyn Fn(web_sys::Event)>);

                        let _ = js_sys::Reflect::set(
                            &recognition,
                            &"onresult".into(),
                            onresult.as_ref().unchecked_ref(),
                        );
                        onresult.forget();

                        // Set up error handler
                        let set_voice_input_active_clone_err = set_voice_input_active;
                        let onerror = wasm_bindgen::closure::Closure::wrap(Box::new(
                            move |_e: web_sys::Event| {
                                set_voice_input_active_clone_err.set(false);
                            },
                        )
                            as Box<dyn Fn(web_sys::Event)>);
                        let _ = js_sys::Reflect::set(
                            &recognition,
                            &"onerror".into(),
                            onerror.as_ref().unchecked_ref(),
                        );
                        onerror.forget();

                        // Set up end handler
                        let set_voice_input_active_clone_end = set_voice_input_active;
                        let onend = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                            set_voice_input_active_clone_end.set(false);
                        })
                            as Box<dyn Fn()>);
                        let _ = js_sys::Reflect::set(
                            &recognition,
                            &"onend".into(),
                            onend.as_ref().unchecked_ref(),
                        );
                        onend.forget();

                        // Start recognition
                        if let Ok(start_fn) = js_sys::Reflect::get(&recognition, &"start".into()) {
                            let _ = js_sys::Reflect::apply(
                                start_fn.unchecked_ref::<js_sys::Function>(),
                                &recognition,
                                &js_sys::Array::new(),
                            );
                        }

                        return;
                    }
                }
            }

            // Fallback if not supported
            gloo_timers::future::TimeoutFuture::new(2000).await;
            set_voice_input_active.set(false);
        });
    };

    // Multiple choice options for easier selection
    let options = match challenge_type {
        ChallengeType::Visual | ChallengeType::Logical => vec![
            ("1", "One"),
            ("2", "Two"),
            ("3", "Three"),
            ("4", "Four"),
            ("5", "Five"),
        ],
        _ => vec![
            ("yes", "Yes"),
            ("no", "No"),
            ("maybe", "Maybe"),
            ("skip", "Skip this challenge"),
        ],
    };

    view! {
        <div class="alternative-inputs bg-yellow-50 dark:bg-yellow-900 border border-yellow-200 dark:border-yellow-700 rounded-lg p-4">
            <h4 class="text-sm font-semibold text-yellow-800 dark:text-yellow-200 mb-3">
                "Alternative Input Methods"
            </h4>

            <div class="space-y-4">
                // Multiple choice options
                <div class="multiple-choice">
                    <p class="text-xs text-yellow-700 dark:text-yellow-300 mb-2">
                        "Select an answer:"
                    </p>
                    <div class="grid grid-cols-2 gap-2">
                        {options.into_iter().map(|(value, label)| {
                            let value_clone = value.to_string();
                            let value_clone_for_class = value_clone.clone();
                            let value_clone_for_click = value_clone.clone();
                            let value_clone_for_aria = value_clone.clone();
                            view! {
                                <button
                                    type="button"
                                    class=move || format!(
                                        "p-2 text-sm rounded border-2 transition-colors {}",
                                        if selected_option.get().as_deref() == Some(&value_clone_for_class) {
                                            "border-yellow-500 bg-yellow-100 dark:bg-yellow-800 text-yellow-900 dark:text-yellow-100"
                                        } else {
                                            "border-yellow-300 dark:border-yellow-600 hover:border-yellow-400 dark:hover:border-yellow-500"
                                        }
                                    )
                                    on:click=move |_| {
                                        set_selected_option.set(Some(value_clone_for_click.clone()));
                                        on_answer.run(value_clone_for_click.clone());
                                    }
                                    aria-pressed=move || selected_option.get().as_deref() == Some(&value_clone_for_aria)
                                >
                                    {label}
                                </button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Voice input
                <div class="voice-input">
                    <Button
                        on_click=Box::new(move || {
                            start_voice_input(leptos::ev::MouseEvent::new("click").unwrap())
                        })
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Small
                        disabled=voice_input_active.get()
                    >
                        {if voice_input_active.get() { "🎤 Listening..." } else { "🎤 Voice Input" }}
                    </Button>
                    <p class="text-xs text-yellow-600 dark:text-yellow-400 mt-1">
                        "Speak your answer clearly"
                    </p>
                </div>
            </div>
        </div>
    }
}

/// Plays audio content, either from a URL or by reading text using SpeechSynthesis.
/// Returns a Future that resolves when playback is finished or fails.
pub async fn play_audio_content(content: String) -> Result<(), ()> {
    if let Some(window) = web_sys::window() {
        // Check if content is a URL or Data URI
        if content.starts_with("http")
            || content.starts_with("/")
            || content.starts_with("data:audio")
        {
            if let Ok(audio) = web_sys::HtmlAudioElement::new_with_src(&content) {
                let promise = js_sys::Promise::new(&mut |resolve, _reject| {
                    // Success handler (ended)
                    let resolve_success = resolve.clone();
                    let on_ended = wasm_bindgen::closure::Closure::once_into_js(move || {
                        let _ = resolve_success.call0(&wasm_bindgen::JsValue::NULL);
                    });
                    let _ = audio.add_event_listener_with_callback(
                        "ended",
                        on_ended.as_ref().unchecked_ref(),
                    );

                    // Error handler
                    let resolve_error = resolve.clone();
                    let on_error = wasm_bindgen::closure::Closure::once_into_js(move || {
                        let _ = resolve_error.call0(&wasm_bindgen::JsValue::NULL);
                    });
                    let _ = audio.add_event_listener_with_callback(
                        "error",
                        on_error.as_ref().unchecked_ref(),
                    );

                    // Play and handle promise rejection
                    let play_promise = audio.play();
                    if let Ok(p) = play_promise {
                        let future = wasm_bindgen_futures::JsFuture::from(p);
                        let resolve_play_error = resolve.clone();
                        spawn_local(async move {
                            if future.await.is_err() {
                                let _ = resolve_play_error.call0(&wasm_bindgen::JsValue::NULL);
                            }
                        });
                    } else {
                        // Play failed immediately
                        let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
                    }
                });
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                return Ok(());
            }
        } else {
            // Use Text-to-Speech
            if let Ok(speech_synthesis) = js_sys::Reflect::get(&window, &"speechSynthesis".into())
                && !speech_synthesis.is_undefined()
            {
                // Safe cast using dyn_into if possible, or unchecked_into if we trust window property
                // Using unchecked_into for now as it's standard for this property,
                // but verifying it's not undefined is good practice.
                let synthesis: web_sys::SpeechSynthesis = speech_synthesis.unchecked_into();

                if let Ok(utterance) = web_sys::SpeechSynthesisUtterance::new_with_text(&content) {
                    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
                        // Success handler
                        let resolve_success = resolve.clone();
                        let on_end = wasm_bindgen::closure::Closure::once_into_js(move || {
                            let _ = resolve_success.call0(&wasm_bindgen::JsValue::NULL);
                        });
                        utterance.set_onend(Some(on_end.as_ref().unchecked_ref()));

                        // Error handler
                        let resolve_error = resolve.clone();
                        let on_error = wasm_bindgen::closure::Closure::once_into_js(move || {
                            let _ = resolve_error.call0(&wasm_bindgen::JsValue::NULL);
                        });
                        utterance.set_onerror(Some(on_error.as_ref().unchecked_ref()));

                        synthesis.speak(&utterance);
                    });
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                    return Ok(());
                }
            }
        }
    }
    // Fallback or error
    Err(())
}
