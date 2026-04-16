use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatTone {
    Neutral,
    Gold,
    Success,
    Warning,
    Danger,
    Info,
}

impl StatTone {
    fn badge_classes(self) -> &'static str {
        match self {
            Self::Neutral => "bg-white/[0.05] text-slate-300 ring-white/10",
            Self::Gold => "bg-gold-500/10 text-gold-400 ring-gold-500/25",
            Self::Success => "bg-success-500/10 text-success-400 ring-success-500/25",
            Self::Warning => "bg-warning-500/10 text-warning-400 ring-warning-500/25",
            Self::Danger => "bg-danger-500/10 text-danger-400 ring-danger-500/25",
            Self::Info => "bg-info-500/10 text-info-400 ring-info-500/25",
        }
    }

    fn accent_border(self) -> &'static str {
        match self {
            Self::Neutral => "border-white/[0.06]",
            Self::Gold => "border-gold-500/30",
            Self::Success => "border-success-500/30",
            Self::Warning => "border-warning-500/30",
            Self::Danger => "border-danger-500/30",
            Self::Info => "border-info-500/30",
        }
    }
}

/// Dashboard stat tile. Callers pass a label, a headline value, an optional
/// delta/caption, an icon, and a tone — the tile takes care of Tailwind
/// classes so the whole dashboard grid stays visually consistent.
#[component]
pub fn StatCard(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(optional, into)] caption: Option<String>,
    #[prop(optional, into)] icon: Option<String>,
    #[prop(default = StatTone::Neutral)] tone: StatTone,
) -> impl IntoView {
    view! {
        <div class=format!(
            "flex items-start gap-4 rounded-2xl border bg-surface-panel p-5 shadow-card {}",
            tone.accent_border()
        )>
            {icon.map(|i| view! {
                <span class=format!(
                    "flex h-11 w-11 shrink-0 items-center justify-center rounded-xl ring-1 {}",
                    tone.badge_classes()
                )>
                    <i class=i></i>
                </span>
            })}
            <div class="min-w-0 flex-1">
                <p class="text-xs font-medium uppercase tracking-wide text-slate-400">{label}</p>
                <p class="mt-1 truncate text-2xl font-bold text-white">{value}</p>
                {caption.map(|c| view! {
                    <p class="mt-1 text-xs text-slate-500">{c}</p>
                })}
            </div>
        </div>
    }
}
