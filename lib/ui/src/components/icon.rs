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
    view! {
        <Icon icon=icon weight=weight size=size_str color=color_str />
    }
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
        "fa-file-lines" => FILE_TEXT,
        "fa-file-pdf" => FILE_PDF,
        "fa-file-signature" => SIGNATURE,
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
        "fa-inbox" => TRAY,
        "fa-keyboard" => KEYBOARD,
        "fa-layer-group" => STACK,
        "fa-life-ring" => LIFEBUOY,
        "fa-list" | "fa-list-ul" => LIST,
        "fa-list-check" | "fa-tasks" => LIST_CHECKS,
        "fa-lock" => LOCK,
        "fa-magnifying-glass" | "fa-search" => MAGNIFYING_GLASS,
        "fa-male" => PERSON,
        "fa-map-location-dot" => MAP_PIN_AREA,
        "fa-minus" => MINUS,
        "fa-paperclip" => PAPERCLIP,
        "fa-paper-plane" => PAPER_PLANE_TILT,
        "fa-phone" => PHONE,
        "fa-plus" => PLUS,
        "fa-plus-circle" => PLUS_CIRCLE,
        "fa-print" => PRINTER,
        "fa-project-diagram" => TREE_STRUCTURE,
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
        "fa-scroll" => SCROLL,
        "fa-shield-alt" => SHIELD,
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
        "fa-user-plus" => USER_PLUS,
        "fa-user-shield" => USER_CHECK,
        "fa-user-tag" => USER_LIST,
        "fa-users" | "fa-users-cog" => USERS,
        "fa-wand-magic-sparkles" => MAGIC_WAND,
        "fa-warehouse" => WAREHOUSE,
        _ => QUESTION,
    }
}
