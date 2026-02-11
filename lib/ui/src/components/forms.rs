//! Form components: Input, Button, Select, Textarea, Checkbox, OtpInput

use crate::core::types::{ButtonSize, ButtonVariant};
use leptos::prelude::*;

// ============================================================================
// BUTTON COMPONENT
// ============================================================================

#[component]
pub fn Button(
    #[prop(optional, into)] class: Option<String>,
    #[prop(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[prop(default = ButtonSize::Medium)] size: ButtonSize,
    #[prop(default = false)] disabled: bool,
    #[prop(default = false)] loading: bool,
    #[prop(default = false)] full_width: bool,
    #[prop(optional)] on_click: Option<Box<dyn Fn()>>,
    #[prop(optional, into)] type_attr: Option<String>,
    children: Children,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let type_attr = type_attr.unwrap_or_else(|| "button".to_string());
    let width_class = if full_width { "w-full" } else { "" };

    let base_class = "inline-flex items-center justify-center font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed";

    let variant_class = match variant {
        ButtonVariant::Primary => {
            "bg-emerald-700 hover:bg-emerald-800 text-white focus:ring-emerald-500"
        }
        ButtonVariant::Secondary => {
            "bg-gray-200 hover:bg-gray-300 text-gray-900 focus:ring-gray-500"
        }
        ButtonVariant::Danger => "bg-red-600 hover:bg-red-700 text-white focus:ring-red-500",
        ButtonVariant::Success => "bg-green-600 hover:bg-green-700 text-white focus:ring-green-500",
        ButtonVariant::Ghost => "hover:bg-gray-100 text-gray-700 focus:ring-gray-500",
    };

    let size_class = match size {
        ButtonSize::Small => "px-3 py-1.5 text-sm rounded",
        ButtonSize::Medium => "px-4 py-2 text-base rounded-md",
        ButtonSize::Large => "px-6 py-3 text-lg rounded-lg",
    };

    let handle_click = move |_| {
        if let Some(ref callback) = on_click {
            callback();
        }
    };

    view! {
        <button
            type=type_attr
            class=format!("{} {} {} {} {} {}", base_class, variant_class, size_class, width_class, class, if loading { "cursor-wait" } else { "" })
            disabled=disabled || loading
            on:click=handle_click
        >
            {loading.then(|| view! {
                <svg class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
            })}
            {children()}
        </button>
    }
}

// ============================================================================
// INPUT COMPONENT
// ============================================================================

#[component]
pub fn Input(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] name: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional, into)] error: Option<String>,
    #[prop(optional, into)] hint: Option<String>,
    #[prop(default = "text".to_string(), into)] input_type: String,
    #[prop(default = false)] required: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_input: Option<Box<dyn Fn(String)>>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let id = id.unwrap_or_else(|| format!("input-{}", name.clone().unwrap_or_default()));
    let has_error = error.is_some();

    let input_class = if has_error {
        "border-red-300 focus:border-red-500 focus:ring-red-500"
    } else {
        "border-gray-300 focus:border-emerald-500 focus:ring-emerald-500"
    };

    let handle_input = move |ev| {
        if let Some(ref callback) = on_input {
            let value = event_target_value(&ev);
            callback(value);
        }
    };

    view! {
        <div class=format!("space-y-1 {}", class)>
            {label.map(|l| view! {
                <label for=id.clone() class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    {l}
                    {required.then(|| view! { <span class="text-red-500 ml-1">"*"</span> })}
                </label>
            })}

            <input
                type=input_type
                id=id
                name=name
                placeholder=placeholder
                value=value
                required=required
                disabled=disabled
                class=format!(
                    "block w-full rounded-md shadow-sm sm:text-sm disabled:bg-gray-100 disabled:cursor-not-allowed {}",
                    input_class
                )
                on:input=handle_input
            />

            {error.map(|e| view! {
                <p class="mt-1 text-sm text-red-600">{e}</p>
            })}

            {hint.map(|h| view! {
                <p class="mt-1 text-sm text-gray-500">{h}</p>
            })}
        </div>
    }
}

// ============================================================================
// OTP INPUT COMPONENT
// ============================================================================

/// OTP Input component for 6-digit verification codes
#[component]
pub fn OtpInput(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] error: Option<String>,
    #[prop(optional, into)] hint: Option<String>,
    value: ReadSignal<String>,
    on_change: WriteSignal<String>,
    #[prop(optional)] on_submit: Option<Box<dyn Fn()>>,
    #[prop(default = false)] disabled: bool,
    #[prop(default = false)] loading: bool,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let id = id.unwrap_or_else(|| "otp-input".to_string());
    let has_error = error.is_some();

    let input_class = if has_error {
        "border-red-300 focus:border-red-500 focus:ring-red-500"
    } else {
        "border-gray-300 focus:border-emerald-500 focus:ring-emerald-500"
    };

    let handle_input = move |ev| {
        let input_value = event_target_value(&ev);
        // Only allow digits and limit to 6 characters
        let filtered: String = input_value
            .chars()
            .filter(|c| c.is_ascii_digit())
            .take(6)
            .collect();
        on_change.set(filtered);
    };

    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter"
            && value.get().len() == 6
            && let Some(ref callback) = on_submit
        {
            callback();
        }
    };

    view! {
        <div class=format!("space-y-2 {}", class)>
            {label.map(|l| view! {
                <label for=id.clone() class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    {l}
                </label>
            })}

            <div class="relative">
                <input
                    type="text"
                    id=id
                    inputmode="numeric"
                    pattern="[0-9]*"
                    autocomplete="one-time-code"
                    placeholder="000000"
                    maxlength="6"
                    prop:value=move || value.get()
                    disabled=disabled || loading
                    class=format!(
                        "block w-full text-center text-2xl font-mono tracking-widest rounded-lg shadow-sm sm:text-xl disabled:bg-gray-100 disabled:cursor-not-allowed px-4 py-3 {}",
                        input_class
                    )
                    on:input=handle_input
                    on:keydown=handle_keydown
                />

                {loading.then(|| view! {
                    <div class="absolute inset-y-0 right-0 flex items-center pr-3">
                        <svg class="animate-spin h-5 w-5 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                    </div>
                })}
            </div>

            // Character counter
            <div class="flex justify-between items-center text-xs">
                <div class=format!(
                    "{}",
                    if value.get().len() == 6 { "text-emerald-600" } else { "text-gray-500" }
                )>
                    {move || format!("{}/6 digits", value.get().len())}
                </div>
                {(value.get().len() == 6).then(|| view! {
                    <div class="text-emerald-600 flex items-center">
                        <svg class="w-4 h-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
                            <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                        </svg>
                        "Ready"
                    </div>
                })}
            </div>

            {error.map(|e| view! {
                <p class="mt-1 text-sm text-red-600 flex items-center">
                    <svg class="w-4 h-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
                    </svg>
                    {e}
                </p>
            })}

            {hint.map(|h| view! {
                <p class="mt-1 text-sm text-gray-500">{h}</p>
            })}
        </div>
    }
}

// ============================================================================
// SELECT COMPONENT
// ============================================================================

#[component]
pub fn Select(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] name: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional, into)] error: Option<String>,
    #[prop(default = false)] required: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_change: Option<Box<dyn Fn(String)>>,
    children: Children,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let id = id.unwrap_or_else(|| format!("select-{}", name.clone().unwrap_or_default()));
    let has_error = error.is_some();

    let select_class = if has_error {
        "border-red-300 focus:border-red-500 focus:ring-red-500"
    } else {
        "border-gray-300 focus:border-emerald-500 focus:ring-emerald-500"
    };

    let handle_change = move |ev| {
        if let Some(ref callback) = on_change {
            let value = event_target_value(&ev);
            callback(value);
        }
    };

    view! {
        <div class=format!("space-y-1 {}", class)>
            {label.map(|l| view! {
                <label for=id.clone() class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    {l}
                    {required.then(|| view! { <span class="text-red-500 ml-1">"*"</span> })}
                </label>
            })}

            <select
                id=id
                name=name
                prop:value=value
                required=required
                disabled=disabled
                class=format!(
                    "block w-full rounded-md shadow-sm sm:text-sm disabled:bg-gray-100 disabled:cursor-not-allowed {}",
                    select_class
                )
                on:change=handle_change
            >
                {children()}
            </select>

            {error.map(|e| view! {
                <p class="mt-1 text-sm text-red-600">{e}</p>
            })}
        </div>
    }
}

// ============================================================================
// TEXTAREA COMPONENT
// ============================================================================

#[component]
pub fn Textarea(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] name: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional, into)] error: Option<String>,
    #[prop(default = 4)] rows: u32,
    #[prop(default = false)] required: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_input: Option<Box<dyn Fn(String)>>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let id = id.unwrap_or_else(|| format!("textarea-{}", name.clone().unwrap_or_default()));
    let has_error = error.is_some();

    let textarea_class = if has_error {
        "border-red-300 focus:border-red-500 focus:ring-red-500"
    } else {
        "border-gray-300 focus:border-emerald-500 focus:ring-emerald-500"
    };

    let handle_input = move |ev| {
        if let Some(ref callback) = on_input {
            let value = event_target_value(&ev);
            callback(value);
        }
    };

    view! {
        <div class=format!("space-y-1 {}", class)>
            {label.map(|l| view! {
                <label for=id.clone() class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    {l}
                    {required.then(|| view! { <span class="text-red-500 ml-1">"*"</span> })}
                </label>
            })}

            <textarea
                id=id
                name=name
                placeholder=placeholder
                rows=rows
                required=required
                disabled=disabled
                class=format!(
                    "block w-full rounded-md shadow-sm sm:text-sm disabled:bg-gray-100 disabled:cursor-not-allowed {}",
                    textarea_class
                )
                on:input=handle_input
            >
                {value}
            </textarea>

            {error.map(|e| view! {
                <p class="mt-1 text-sm text-red-600">{e}</p>
            })}
        </div>
    }
}

// ============================================================================
// CHECKBOX COMPONENT
// ============================================================================

#[component]
pub fn Checkbox(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] name: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(default = false)] checked: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_change: Option<Box<dyn Fn(bool)>>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let id = id.unwrap_or_else(|| format!("checkbox-{}", name.clone().unwrap_or_default()));

    let handle_change = move |ev| {
        if let Some(ref callback) = on_change {
            let checked = event_target_checked(&ev);
            callback(checked);
        }
    };

    view! {
        <div class=format!("flex items-center {}", class)>
            <input
                type="checkbox"
                id=id.clone()
                name=name
                checked=checked
                disabled=disabled
                class="h-4 w-4 rounded border-gray-300 text-emerald-600 focus:ring-emerald-500 disabled:cursor-not-allowed"
                on:change=handle_change
            />
            {label.map(|l| view! {
                <label for=id class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
                    {l}
                </label>
            })}
        </div>
    }
}

// ============================================================================
// RADIO COMPONENT
// ============================================================================

#[component]
pub fn Radio(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(into)] name: String,
    #[prop(into)] value: String,
    #[prop(optional, into)] label: Option<String>,
    #[prop(default = false)] checked: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_change: Option<Box<dyn Fn(String)>>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let id = id.unwrap_or_else(|| format!("radio-{}-{}", name, value));

    let value_clone = value.clone();
    let handle_change = move |_| {
        if let Some(ref callback) = on_change {
            callback(value_clone.clone());
        }
    };

    view! {
        <div class=format!("flex items-center {}", class)>
            <input
                type="radio"
                id=id.clone()
                name=name
                value=value
                checked=checked
                disabled=disabled
                class="h-4 w-4 border-gray-300 text-emerald-600 focus:ring-emerald-500 disabled:cursor-not-allowed"
                on:change=handle_change
            />
            {label.map(|l| view! {
                <label for=id class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
                    {l}
                </label>
            })}
        </div>
    }
}

// ============================================================================
// RADIO GROUP COMPONENT
// ============================================================================

#[derive(Clone, Debug)]
pub struct RadioOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

#[component]
pub fn RadioGroup(
    #[prop(optional, into)] class: Option<String>,
    #[prop(into)] name: String,
    #[prop(optional, into)] label: Option<String>,
    #[prop(into)] options: Vec<RadioOption>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional, into)] error: Option<String>,
    #[prop(default = false)] horizontal: bool,
    #[prop(optional)] on_change: Option<Box<dyn Fn(String)>>,
) -> impl IntoView {
    use std::rc::Rc;
    let class = class.unwrap_or_default();
    let on_change_rc = Rc::new(on_change);
    let layout_class = if horizontal {
        "flex flex-row space-x-4"
    } else {
        "flex flex-col space-y-2"
    };

    view! {
        <div class=format!("space-y-2 {}", class)>
            {label.map(|l| view! {
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    {l}
                </label>
            })}

            <div class=layout_class role="radiogroup">
                {options.into_iter().map(|opt| {
                    let is_checked = value.as_ref().map(|v| v == &opt.value).unwrap_or(false);
                    let on_change = Rc::clone(&on_change_rc);

                    view! {
                        <Radio
                            name=name.clone()
                            value=opt.value
                            label=opt.label
                            checked=is_checked
                            disabled=opt.disabled
                            on_change=Box::new(move |v| {
                                if let Some(ref callback) = *on_change {
                                    callback(v);
                                }
                            })
                        />
                    }
                }).collect_view()}
            </div>

            {error.map(|e| view! {
                <p class="mt-1 text-sm text-red-600">{e}</p>
            })}
        </div>
    }
}

// ============================================================================
// SWITCH/TOGGLE COMPONENT
// ============================================================================

#[component]
pub fn Switch(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] name: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(default = false)] checked: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_change: Option<Box<dyn Fn(bool)>>,
) -> impl IntoView {
    use std::rc::Rc;
    let class = class.unwrap_or_default();
    let id = id.unwrap_or_else(|| format!("switch-{}", name.clone().unwrap_or_default()));
    let on_change_rc = Rc::new(on_change);

    view! {
        <div class=format!("flex items-center {}", class)>
            <button
                type="button"
                role="switch"
                aria-checked=checked
                id=id.clone()
                disabled=disabled
                class=format!(
                    "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 {}",
                    if checked { "bg-emerald-600" } else { "bg-gray-200 dark:bg-gray-700" }
                )
                on:click={
                    let on_change = Rc::clone(&on_change_rc);
                    move |_| {
                        if !disabled
                            && let Some(ref callback) = *on_change {
                                callback(!checked);
                            }
                    }
                }
            >
                <span
                    class=format!(
                        "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {}",
                        if checked { "translate-x-5" } else { "translate-x-0" }
                    )
                ></span>
            </button>
            {label.map(|l| view! {
                <label for=id class="ml-3 text-sm text-gray-700 dark:text-gray-300">
                    {l}
                </label>
            })}
        </div>
    }
}

// ============================================================================
// FILE INPUT COMPONENT
// ============================================================================

#[component]
pub fn FileInput(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] name: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] accept: Option<String>,
    #[prop(default = false)] multiple: bool,
    #[prop(default = false)] required: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(optional, into)] error: Option<String>,
    #[prop(optional)] on_change: Option<Box<dyn Fn(Vec<web_sys::File>)>>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let id = id.unwrap_or_else(|| format!("file-{}", name.clone().unwrap_or_default()));
    let has_error = error.is_some();

    use wasm_bindgen::JsCast;

    let handle_change = move |ev: web_sys::Event| {
        if let Some(ref callback) = on_change {
            let target = ev
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
            if let Some(input) = target
                && let Some(files) = input.files()
            {
                let mut file_list = Vec::new();
                for i in 0..files.length() {
                    if let Some(file) = files.get(i) {
                        file_list.push(file);
                    }
                }
                callback(file_list);
            }
        }
    };

    view! {
        <div class=format!("space-y-1 {}", class)>
            {label.map(|l| view! {
                <label for=id.clone() class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    {l}
                    {required.then(|| view! { <span class="text-red-500 ml-1">"*"</span> })}
                </label>
            })}

            <input
                type="file"
                id=id
                name=name
                accept=accept
                multiple=multiple
                required=required
                disabled=disabled
                class=format!(
                    "block w-full text-sm text-gray-900 dark:text-gray-100 border rounded-md cursor-pointer bg-gray-50 dark:bg-gray-700 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50 {}",
                    if has_error { "border-red-300" } else { "border-gray-300 dark:border-gray-600" }
                )
                on:change=handle_change
            />

            {error.map(|e| view! {
                <p class="mt-1 text-sm text-red-600">{e}</p>
            })}
        </div>
    }
}

// ============================================================================
// FORM GROUP COMPONENT
// ============================================================================

#[component]
pub fn FormGroup(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(default = false)] required: bool,
    children: Children,
) -> impl IntoView {
    let class = class.unwrap_or_default();

    view! {
        <div class=format!("space-y-2 {}", class)>
            {label.map(|l| view! {
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    {l}
                    {required.then(|| view! { <span class="text-red-500 ml-1">"*"</span> })}
                </label>
            })}
            {children()}
        </div>
    }
}

// ============================================================================
// DATE PICKER COMPONENT (Enhanced)
// ============================================================================

/// Enhanced date picker with validation and min/max support
#[component]
pub fn DatePicker(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] name: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional, into)] error: Option<String>,
    #[prop(optional, into)] hint: Option<String>,
    #[prop(optional, into)] min: Option<String>,
    #[prop(optional, into)] max: Option<String>,
    #[prop(default = false)] required: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_change: Option<Box<dyn Fn(String)>>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let id = id.unwrap_or_else(|| format!("date-{}", name.clone().unwrap_or_default()));
    let has_error = error.is_some();

    let input_class = if has_error {
        "border-red-300 focus:border-red-500 focus:ring-red-500"
    } else {
        "border-gray-300 focus:border-emerald-500 focus:ring-emerald-500"
    };

    let handle_change = move |ev| {
        if let Some(ref callback) = on_change {
            let value = event_target_value(&ev);
            callback(value);
        }
    };

    view! {
        <div class=format!("space-y-1 {}", class)>
            {label.map(|l| view! {
                <label for=id.clone() class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    {l}
                    {required.then(|| view! { <span class="text-red-500 ml-1">"*"</span> })}
                </label>
            })}

            <input
                type="date"
                id=id
                name=name
                prop:value=value
                min=min
                max=max
                required=required
                disabled=disabled
                class=format!(
                    "block w-full rounded-md shadow-sm sm:text-sm disabled:bg-gray-100 disabled:cursor-not-allowed {}",
                    input_class
                )
                on:change=handle_change
            />

            {error.map(|e| view! {
                <p class="mt-1 text-sm text-red-600 flex items-center">
                    <svg class="w-4 h-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
                    </svg>
                    {e}
                </p>
            })}

            {hint.map(|h| view! {
                <p class="mt-1 text-sm text-gray-500">{h}</p>
            })}
        </div>
    }
}

// ============================================================================
// FILE UPLOAD COMPONENT (Enhanced with preview)
// ============================================================================

/// Enhanced file upload with preview and drag-and-drop support
#[component]
pub fn FileUpload(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] name: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] accept: Option<String>,
    #[prop(optional, into)] error: Option<String>,
    #[prop(optional, into)] hint: Option<String>,
    #[prop(default = false)] multiple: bool,
    #[prop(default = false)] required: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(default = false)] show_preview: bool,
    #[prop(optional)] on_change: Option<Box<dyn Fn(Vec<web_sys::File>)>>,
) -> impl IntoView {
    use wasm_bindgen::JsCast;

    let class = class.unwrap_or_default();
    let id = id.unwrap_or_else(|| format!("upload-{}", name.clone().unwrap_or_default()));
    let has_error = error.is_some();

    let (files, set_files) = signal::<Vec<String>>(Vec::new());

    let handle_change = move |ev: web_sys::Event| {
        if let Some(target) = ev.target() {
            if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                if let Some(file_list) = input.files() {
                    let mut files_vec = Vec::new();
                    let mut file_names = Vec::new();

                    for i in 0..file_list.length() {
                        if let Some(file) = file_list.get(i) {
                            file_names.push(file.name());
                            files_vec.push(file);
                        }
                    }

                    set_files.set(file_names);

                    if let Some(ref callback) = on_change {
                        callback(files_vec);
                    }
                }
            }
        }
    };

    view! {
        <div class=format!("space-y-2 {}", class)>
            {label.map(|l| view! {
                <label for=id.clone() class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    {l}
                    {required.then(|| view! { <span class="text-red-500 ml-1">"*"</span> })}
                </label>
            })}

            <div
                class=format!(
                    "relative border-2 border-dashed rounded-lg p-6 transition-colors {}",
                    if has_error {
                        "border-red-300 bg-red-50 dark:bg-red-900/20"
                    } else {
                        "border-gray-300 dark:border-gray-600 hover:border-emerald-400"
                    }
                )
            >
                <input
                    type="file"
                    id=id.clone()
                    name=name
                    accept=accept.clone()
                    multiple=multiple
                    required=required
                    disabled=disabled
                    class="sr-only"
                    on:change=handle_change
                />

                <label
                    for=id
                    class=format!(
                        "flex flex-col items-center justify-center cursor-pointer {}",
                        if disabled { "opacity-50 cursor-not-allowed" } else { "" }
                    )
                >
                    <svg class="w-12 h-12 text-gray-400 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"/>
                    </svg>
                    <p class="text-sm text-gray-600 dark:text-gray-400 mb-1">
                        <span class="font-semibold text-emerald-600 dark:text-emerald-400">"Click to upload"</span>
                    </p>
                    <p class="text-xs text-gray-500 dark:text-gray-500">
                        {if let Some(ref a) = accept {
                            format!("Accepted: {}", a)
                        } else {
                            "Any file type".to_string()
                        }}
                    </p>
                </label>
            </div>

            // File list preview
            {show_preview.then(|| view! {
                <div class="space-y-2">
                    {move || files.get().into_iter().map(|file_name| view! {
                        <div class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800 rounded-md">
                            <div class="flex items-center space-x-2">
                                <svg class="w-5 h-5 text-gray-400" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z" clip-rule="evenodd"/>
                                </svg>
                                <span class="text-sm text-gray-700 dark:text-gray-300 truncate">
                                    {file_name}
                                </span>
                            </div>
                            <svg class="w-5 h-5 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                            </svg>
                        </div>
                    }).collect_view()}
                </div>
            })}

            {error.map(|e| view! {
                <p class="mt-1 text-sm text-red-600 flex items-center">
                    <svg class="w-4 h-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
                    </svg>
                    {e}
                </p>
            })}

            {hint.map(|h| view! {
                <p class="mt-1 text-sm text-gray-500">{h}</p>
            })}
        </div>
    }
}
