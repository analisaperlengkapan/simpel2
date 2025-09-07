use leptos::*;
use leptos_router::*;

#[component]
pub fn Header() -> impl IntoView {
    let (user_name, _) = signal("Admin".to_string());

    let logout = move |_| {
        let window = web_sys::window().unwrap();
        let _ = window.location().set_href("/antarmuka/portal-leptos");
    };

    view! {
        <header class="header">
            <div class="header-content">
                <div class="header-logo">
                    <h2><i class="fas fa-tools"></i> " Perlengkapan SIMPelv2"</h2>
                </div>
                <nav class="header-nav">
                    <A href="/" class="nav-link">"Dashboard"</A>
                    <A href="/barang" class="nav-link">"Barang"</A>
                    <A href="/pengadaan" class="nav-link">"Pengadaan"</A>
                    <A href="/laporan" class="nav-link">"Laporan"</A>
                </nav>
                <div class="header-user">
                    <span class="user-name">{user_name}</span>
                    <button class="logout-button" on:click=logout>
                        <i class="fas fa-sign-out-alt"></i> " Logout"
                    </button>
                </div>
            </div>
        </header>
    }
}

#[component]
pub fn StatCard(icon: &'static str, title: String, value: String) -> impl IntoView {
    view! {
        <div class="stat-card">
            <div class="stat-icon">
                <i class=icon></i>
            </div>
            <div class="stat-info">
                <h3>{title}</h3>
                <p class="stat-number">{value}</p>
            </div>
        </div>
    }
}

#[component]
pub fn SearchBox(#[prop(into)] on_search: Callback<String>) -> impl IntoView {
    let (search_value, set_search_value) = signal(String::new());

    let handle_search = move |_| {
    on_search.run(search_value.get());
    };

    let handle_input = move |ev| {
        let value = event_target_value(&ev);
        set_search_value.set(value);
    };

    view! {
        <div class="search-box">
            <input
                type="text"
                placeholder="Cari..."
                value=move || search_value.get()
                on:input=handle_input
            />
            <button class="search-btn" on:click=handle_search>
                <i class="fas fa-search"></i>
            </button>
        </div>
    }
}

#[component]
pub fn ActionButton(
    #[prop(into)] text: String,
    #[prop(into)] icon: &'static str,
    #[prop(into)] class: &'static str,
    #[prop(into)] on_click: Callback<()>
) -> impl IntoView {
    view! {
    <button class=class on:click=move |_| on_click.run(())>
            <i class=icon></i> {text}
        </button>
    }
}

#[component]
pub fn StatusBadge(#[prop(into)] status: String) -> impl IntoView {
    let badge_class = match status.as_str() {
        "Tersedia" => "status-badge status-tersedia",
        "Habis" => "status-badge status-habis",
        "Perbaikan" => "status-badge status-perbaikan",
        _ => "status-badge status-tersedia",
    };

    view! {
        <span class=badge_class>{status}</span>
    }
}

#[component]
pub fn FormGroup(
    #[prop(into)] label: String,
    #[prop(into)] input_type: &'static str,
    #[prop(into)] placeholder: String,
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_change: Callback<String>
) -> impl IntoView {
    let handle_input = move |ev| {
        let value = event_target_value(&ev);
    on_change.run(value);
    };

    view! {
        <div class="form-group">
            <label>{label}</label>
            <input
                type=input_type
                placeholder=placeholder
                value=move || value.get()
                on:input=handle_input
            />
        </div>
    }
}

#[component]
pub fn FormSelect(
    #[prop(into)] label: String,
    #[prop(into)] options: Vec<(String, String)>,
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_change: Callback<String>
) -> impl IntoView {
    let handle_change = move |ev| {
        let value = event_target_value(&ev);
    on_change.run(value);
    };

    view! {
        <div class="form-group">
            <label>{label}</label>
            <select value=move || value.get() on:change=handle_change>
                <option value="">"Pilih..."</option>
                {options.into_iter().map(|(value, text)| {
                    view! {
                        <option value=value>{text}</option>
                    }
                }).collect::<Vec<_>>()}
            </select>
        </div>
    }
}
