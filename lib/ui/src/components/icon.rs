//! Unified icon component backed by [`phosphor_leptos`].
//!
//! ```ignore
//! use lib_ui::prelude::*;
//! use phosphor_leptos::USER_CIRCLE;
//!
//! view! { <AppIcon icon=USER_CIRCLE size=20 /> }
//! ```
//!
//! Replaces `<i class="fas fa-*">` FontAwesome usage. New code must use
//! the typed phosphor constants; see the crate root of `phosphor_leptos`
//! for the full catalogue.
//!
//! For call-sites that still carry a FontAwesome class string in data
//! (e.g. menu configs), [`icon_from_fa_class`] resolves the class at
//! runtime to the nearest phosphor counterpart. Unknown classes fall
//! back to [`phosphor_leptos::QUESTION`].

use leptos::prelude::*;
use phosphor_leptos::{Icon, IconData, IconWeight};

pub use phosphor_leptos::IconWeight as AppIconWeight;

#[component]
pub fn AppIcon(
    #[prop(into)] icon: IconData,
    #[prop(optional, default = 16u32)] size: u32,
    #[prop(optional, default = IconWeight::Regular)] weight: IconWeight,
    #[prop(optional, into)] color: Option<String>,
) -> impl IntoView {
    let size_str = format!("{}px", size);
    let color_str = color.unwrap_or_else(|| "currentColor".to_string());
    view! { <Icon icon=icon weight=weight size=size_str color=color_str /> }
}

/// Resolve a FontAwesome class string to the nearest phosphor icon.
///
/// Accepts `"fas fa-user"`, `"far fa-clock"`, `"fab fa-..."`, or just
/// `"fa-user"`. Extra Tailwind classes after the icon name are ignored.
/// Unknown icons fall back to [`phosphor_leptos::QUESTION`].
pub fn icon_from_fa_class(class: &str) -> IconData {
    use phosphor_leptos::*;

    let trimmed = class.trim();
    let core = trimmed
        .strip_prefix("fas ")
        .or_else(|| trimmed.strip_prefix("far "))
        .or_else(|| trimmed.strip_prefix("fab "))
        .unwrap_or(trimmed);
    let name = core.split_whitespace().next().unwrap_or(core);

    match name {
        "fa-arrow-left" => ARROW_LEFT,
        "fa-arrow-right" => ARROW_RIGHT,
        "fa-balance-scale" => SCALES,
        "fa-ban" => PROHIBIT,
        "fa-barcode" => BARCODE,
        "fa-bars" => LIST,
        "fa-book" => BOOK,
        "fa-book-open" => BOOK_OPEN,
        "fa-box" | "fa-box-open" => PACKAGE,
        "fa-boxes" | "fa-boxes-stacked" => STACK,
        "fa-building" | "fa-building-circle-exclamation" => BUILDING,
        "fa-calendar" => CALENDAR,
        "fa-calendar-alt" | "fa-calendar-days" => CALENDAR_BLANK,
        "fa-calendar-check" => CALENDAR_CHECK,
        "fa-calendar-times" | "fa-calendar-xmark" => CALENDAR_X,
        "fa-chart-bar" | "fa-chart-simple" => CHART_BAR,
        "fa-chart-line" => CHART_LINE,
        "fa-chart-pie" => CHART_PIE,
        "fa-check" => CHECK,
        "fa-check-circle" => CHECK_CIRCLE,
        "fa-check-double" => CHECKS,
        "fa-check-square" => CHECK_SQUARE,
        "fa-chevron-down" => CARET_DOWN,
        "fa-chevron-left" => CARET_LEFT,
        "fa-chevron-right" => CARET_RIGHT,
        "fa-chevron-up" => CARET_UP,
        "fa-circle" => CIRCLE,
        "fa-circle-exclamation" | "fa-exclamation-circle" => WARNING_CIRCLE,
        "fa-circle-info" | "fa-info-circle" | "fa-info" => INFO,
        "fa-circle-question" | "fa-question-circle" => QUESTION,
        "fa-clipboard" => CLIPBOARD,
        "fa-clipboard-check" | "fa-clipboard-list" => CLIPBOARD_TEXT,
        "fa-clock" => CLOCK,
        "fa-clock-rotate-left" | "fa-history" => CLOCK_COUNTER_CLOCKWISE,
        "fa-cloud-arrow-up" => CLOUD_ARROW_UP,
        "fa-code" => CODE,
        "fa-code-branch" => GIT_BRANCH,
        "fa-cog" => GEAR,
        "fa-cogs" => GEAR_SIX,
        "fa-cube" => CUBE,
        "fa-database" => DATABASE,
        "fa-download" => DOWNLOAD_SIMPLE,
        "fa-edit" | "fa-pen" => PENCIL_SIMPLE,
        "fa-ellipsis-h" => DOTS_THREE,
        "fa-envelope" => ENVELOPE,
        "fa-eraser" => ERASER,
        "fa-exclamation-triangle" | "fa-triangle-exclamation" => WARNING,
        "fa-eye" => EYE,
        "fa-file-arrow-down" => FILE_ARROW_DOWN,
        "fa-file-arrow-up" | "fa-file-import" => FILE_ARROW_UP,
        "fa-file-circle-check" => FILE_TEXT,
        "fa-file-excel" => FILE_XLS,
        "fa-file-circle-minus" => FILE_MINUS,
        "fa-file-lines" | "fa-file-alt" => FILE_TEXT,
        "fa-file-pdf" => FILE_PDF,
        "fa-file-signature" | "fa-signature" => SIGNATURE,
        "fa-file-word" => FILE_DOC,
        "fa-filter" => FUNNEL,
        "fa-flag-checkered" => FLAG_CHECKERED,
        "fa-folder-open" => FOLDER_OPEN,
        "fa-forward" => FAST_FORWARD,
        "fa-gauge-high" => GAUGE,
        "fa-hand-pointer" => HAND_POINTING,
        "fa-handshake" => HANDSHAKE,
        "fa-headset" => HEADSET,
        "fa-heart-pulse" => HEARTBEAT,
        "fa-home" => HOUSE,
        "fa-hourglass-half" => HOURGLASS,
        "fa-id-badge" => IDENTIFICATION_BADGE,
        "fa-inbox" => TRAY,
        "fa-keyboard" => KEYBOARD,
        "fa-layer-group" => STACK,
        "fa-life-ring" => LIFEBUOY,
        "fa-list" | "fa-list-ul" => LIST,
        "fa-list-check" | "fa-tasks" => LIST_CHECKS,
        "fa-lock" => LOCK,
        "fa-magnifying-glass" | "fa-search" => MAGNIFYING_GLASS,
        "fa-male" => PERSON,
        "fa-map" => MAP_TRIFOLD,
        "fa-map-location-dot" => MAP_PIN_AREA,
        "fa-minus" => MINUS,
        "fa-paperclip" => PAPERCLIP,
        "fa-paper-plane" => PAPER_PLANE_TILT,
        "fa-phone" => PHONE,
        "fa-plus" => PLUS,
        "fa-plus-circle" => PLUS_CIRCLE,
        "fa-print" => PRINTER,
        "fa-project-diagram" | "fa-diagram-project" => TREE_STRUCTURE,
        "fa-qrcode" => QR_CODE,
        "fa-random" => SHUFFLE,
        "fa-redo" | "fa-rotate-right" | "fa-sync-alt" => ARROW_CLOCKWISE,
        "fa-repeat" => REPEAT,
        "fa-right-to-bracket" | "fa-sign-in-alt" => SIGN_IN,
        "fa-road" => PATH,
        "fa-rotate-left" | "fa-undo" => ARROW_COUNTER_CLOCKWISE,
        "fa-ruler" => RULER,
        "fa-sack-dollar" => MONEY_WAVY,
        "fa-save" => FLOPPY_DISK,
        "fa-screwdriver-wrench" => WRENCH,
        "fa-scroll" => SCROLL,
        "fa-shield-alt" => SHIELD,
        "fa-shirt" => T_SHIRT,
        "fa-shoe-prints" => FOOTPRINTS,
        "fa-sign-out-alt" => SIGN_OUT,
        "fa-sliders" => SLIDERS,
        "fa-spinner" => SPINNER,
        "fa-stamp" => STAMP,
        "fa-stream" => FLOW_ARROW,
        "fa-table" => TABLE,
        "fa-tags" => TAG,
        "fa-times" | "fa-xmark" => X,
        "fa-times-circle" => X_CIRCLE,
        "fa-trash" | "fa-trash-alt" | "fa-trash-can" => TRASH,
        "fa-trophy" => TROPHY,
        "fa-tshirt" => T_SHIRT,
        "fa-upload" => UPLOAD_SIMPLE,
        "fa-user" | "fa-user-tie" => USER,
        "fa-user-check" => USER_CHECK,
        "fa-user-plus" => USER_PLUS,
        "fa-user-shield" => USER_CHECK,
        "fa-user-tag" => USER_LIST,
        "fa-users" | "fa-users-cog" => USERS,
        "fa-wand-magic-sparkles" => MAGIC_WAND,
        "fa-warehouse" => WAREHOUSE,
        "fa-wave-square" => WAVE_SQUARE,
        _ => QUESTION,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    /// Every `fa-…` name the apps and libraries hand over in a class string (`"fas fa-x"`,
    /// `"far fa-x"`, `"fab fa-x"` or a bare `"fa-x"`), with the file using it. Only the first
    /// name of the string counts, as in [`icon_from_fa_class`].
    fn nama_fa_di(dir: &Path, hasil: &mut Vec<(String, PathBuf)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let nama = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if nama != "target" && nama != "node_modules" && !nama.starts_with('.') {
                    nama_fa_di(&path, hasil);
                }
            } else if path.extension().is_some_and(|e| e == "rs")
                && !path.ends_with("lib/ui/src/components/icon.rs")
            {
                let Ok(sumber) = std::fs::read_to_string(&path) else {
                    continue;
                };
                for awalan in ["\"fas fa-", "\"far fa-", "\"fab fa-", "\"fa-"] {
                    for (i, _) in sumber.match_indices(awalan) {
                        let dari = i + awalan.len() - "fa-".len();
                        let nama: String = sumber[dari..]
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                            .collect();
                        // `fa-spin` and its kin are CSS modifiers, not icons, and a bare `"fa-"`
                        // is a prefix being tested for, not a name.
                        let modifier = matches!(
                            nama.as_str(),
                            "fa-spin"
                                | "fa-pulse"
                                | "fa-fw"
                                | "fa-xs"
                                | "fa-sm"
                                | "fa-lg"
                                | "fa-2x"
                                | "fa-3x"
                        );
                        if nama.len() > "fa-".len() && !modifier {
                            hasil.push((nama, path.clone()));
                        }
                    }
                }
            }
        }
    }

    /// An unknown name silently becomes a question mark. That is how the dashboard tiles
    /// "Izin Pemakaian" and "Pakaian Dinas Menunggu" and the admin menu entries
    /// "Template Dokumen" and "Monitoring Workflow" went out showing "?": the name was written
    /// at the call site and nothing checked it against this table. The list of names is read
    /// from the sources, so a new icon name cannot be forgotten.
    #[test]
    fn every_icon_name_the_apps_use_resolves_to_an_icon() {
        let akar = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut dipakai = Vec::new();
        for sub in ["antarmuka", "lib"] {
            nama_fa_di(&akar.join(sub), &mut dipakai);
        }
        // The scan itself must work: a moved directory would otherwise pass this vacuously.
        assert!(
            dipakai.len() > 50,
            "the scan found only {} icon names; did the source layout move?",
            dipakai.len()
        );
        let tanda_tanya = icon_from_fa_class("fa-circle-question");
        let mut tak_dikenal: Vec<String> = dipakai
            .iter()
            .filter(|(n, _)| {
                *icon_from_fa_class(n) == *tanda_tanya
                    && n != "fa-circle-question"
                    && n != "fa-question-circle"
            })
            .map(|(n, p)| format!("{n} ({})", p.display()))
            .collect();
        tak_dikenal.sort();
        tak_dikenal.dedup();
        assert!(
            tak_dikenal.is_empty(),
            "icon names with no mapping in icon_from_fa_class (they render as '?'): {tak_dikenal:?}"
        );
    }
}
