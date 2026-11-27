//! The current calendar year, read from the clock.
//!
//! A year written into the source is a date the screen silently expires on: the Kebutuhan BMN form
//! defaulted to 2026 and its year lists ended at 2027/2028, the footer said "2026", and from 1 January
//! 2027 each of them would have been wrong with nothing failing. Everything that needs "this year"
//! asks here.

use chrono::Datelike;

/// The year it is now, in the reader's own time zone (a campaign for the new year opens at the
/// reader's midnight, not at UTC's).
pub fn tahun_berjalan() -> i32 {
    chrono::Local::now().year()
}
