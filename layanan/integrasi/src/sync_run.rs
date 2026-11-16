//! Every sync run leaves its outcome in `integrasi.sync_status`.
//!
//! The table has existed since migration 002, with a row per source, a status column, an error
//! column and counters. Nothing ever wrote to it: on staging all six rows still said `pending`
//! after months of runs. A failed run left no trace outside the pod log (kept one day, readable only
//! with cluster access), and `integrasi.api_log` only records calls that got an HTTP answer, so a
//! run that died on DNS, on its token, or before its first request did not appear anywhere a person
//! could look.
//!
//! What is recorded, per row of the run's module:
//!
//! * `running` when the run starts, with the CronJob's own schedule and the next time it should
//!   fire (`SYNC_SCHEDULE` / `SYNC_TIMEZONE`, set by the chart from the CronJob spec, so the
//!   expectation cannot drift from the schedule that actually runs);
//! * `completed`, `last_success_at`, and the number of rows now in the table the application reads
//!   (not a count the fetcher reports about itself; see `RECORDS_NOW`), or
//! * `failed` and the reason, shortened to `MAX_ERROR_CHARS`.
//!
//! A run that is killed between the two (OOM, deadline, node loss) stays `running`; the next
//! scheduled time then passes without a newer start, which is how a reader sees it.

use chrono::{DateTime, Utc};
use std::str::FromStr;
use tokio_postgres::Client;

use crate::scheduler::DataSource;

/// Longest reason kept in `last_sync_error`. Enough for the chain of an error; a whole upstream
/// HTML error page is not useful on an admin screen.
pub const MAX_ERROR_CHARS: usize = 1000;

/// The `module` of the `sync_status` rows a source's run speaks for (seeded by migration 002).
pub fn module_of(source: DataSource) -> Option<&'static str> {
    match source {
        DataSource::Mysimkari => Some("MySIMKARI"),
        DataSource::Siman => Some("SIMAN"),
        DataSource::Monsakti => Some("MonSAKTI_ADM"),
        DataSource::All => None,
    }
}

/// The schedule of the CronJob this run belongs to.
#[derive(Debug, Clone)]
pub struct Schedule {
    expression: String,
    timezone: chrono_tz::Tz,
    cron: croner::Cron,
}

impl Schedule {
    /// `SYNC_SCHEDULE` (five-field cron, as in the CronJob spec) and `SYNC_TIMEZONE` (IANA name,
    /// UTC when unset, as Kubernetes does). `None` when the run was not started by a CronJob
    /// (a developer's shell), or when either value cannot be understood.
    pub fn from_env() -> Option<Self> {
        let expression = std::env::var("SYNC_SCHEDULE").ok()?;
        let timezone = std::env::var("SYNC_TIMEZONE").unwrap_or_else(|_| "UTC".to_string());
        match Self::parse(&expression, &timezone) {
            Ok(s) => Some(s),
            Err(e) => {
                tracing::error!(
                    "SYNC_SCHEDULE/SYNC_TIMEZONE not usable, next run not recorded: {e}"
                );
                None
            }
        }
    }

    pub fn parse(expression: &str, timezone: &str) -> Result<Self, String> {
        let cron = croner::Cron::from_str(expression)
            .map_err(|e| format!("schedule {expression:?}: {e}"))?;
        let timezone =
            chrono_tz::Tz::from_str(timezone).map_err(|e| format!("timezone {timezone:?}: {e}"))?;
        Ok(Self {
            expression: expression.to_string(),
            timezone,
            cron,
        })
    }

    /// The first time after `after` at which the CronJob fires.
    pub fn next_after(&self, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
        self.cron
            .find_next_occurrence(&after.with_timezone(&self.timezone), false)
            .ok()
            .map(|t| t.with_timezone(&Utc))
    }

    /// What `sync_frequency` holds: the schedule as the CronJob states it.
    pub fn describe(&self) -> String {
        format!("{} {}", self.expression, self.timezone.name())
    }
}

/// `text` cut to at most `MAX_ERROR_CHARS` characters, on a character boundary.
pub fn shorten(text: &str) -> String {
    match text.char_indices().nth(MAX_ERROR_CHARS) {
        Some((at, _)) => format!("{}…", &text[..at]),
        None => text.to_string(),
    }
}

/// Rows in the table each `sync_status` row stands for, read AFTER the run. These are the tables
/// perlengkapan reads (`shared/pegawai_ref.rs`, bank aset); a row the fetcher believes it wrote but
/// that is not there is not counted.
const RECORDS_NOW: &str = "CASE endpoint \
     WHEN 'satker' THEN (SELECT count(*) FROM integrasi.mysimkari_satker)::integer \
     WHEN 'pegawai' THEN (SELECT count(*) FROM integrasi.mysimkari_pegawai \
                          WHERE status_pegawai = 'aktif')::integer \
     WHEN 'aset' THEN (SELECT count(*) FROM integrasi.siman_aset)::integer \
     ELSE last_sync_records_count END";

/// Marks the module's rows `running`. Returns how many rows it marked; 0 means the module has no
/// row (migration 002 not applied), which the caller reports.
pub async fn mark_started(
    db: &Client,
    module: &str,
    schedule: Option<&Schedule>,
) -> Result<u64, tokio_postgres::Error> {
    let next = schedule.and_then(|s| s.next_after(Utc::now()));
    let frequency = schedule.map(Schedule::describe);
    db.execute(
        "UPDATE integrasi.sync_status \
            SET last_sync_status = 'running', \
                last_sync_started_at = now(), \
                sync_frequency = COALESCE($2, sync_frequency), \
                next_sync_scheduled_at = COALESCE($3, next_sync_scheduled_at), \
                is_enabled = TRUE \
          WHERE module = $1",
        &[&module, &frequency, &next],
    )
    .await
}

/// Records how the run ended. `outcome` is `Err(reason)` for a failed run.
pub async fn mark_finished(
    db: &Client,
    module: &str,
    outcome: Result<(), String>,
) -> Result<u64, tokio_postgres::Error> {
    match outcome {
        Ok(()) => {
            db.execute(
                &format!(
                    "UPDATE integrasi.sync_status \
                        SET last_sync_status = 'completed', \
                            last_sync_completed_at = now(), \
                            last_success_at = now(), \
                            last_sync_error = NULL, \
                            last_sync_records_count = {RECORDS_NOW}, \
                            total_syncs_completed = COALESCE(total_syncs_completed, 0) + 1 \
                      WHERE module = $1"
                ),
                &[&module],
            )
            .await
        }
        Err(reason) => {
            let reason = shorten(&reason);
            db.execute(
                "UPDATE integrasi.sync_status \
                    SET last_sync_status = 'failed', \
                        last_sync_completed_at = now(), \
                        last_sync_error = $2, \
                        total_syncs_failed = COALESCE(total_syncs_failed, 0) + 1 \
                  WHERE module = $1",
                &[&module, &reason],
            )
            .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn the_next_run_follows_the_cronjob_schedule_in_its_timezone() {
        // MySIMKARI: every six hours, Jakarta time (UTC+7, no DST).
        let s = Schedule::parse("0 */6 * * *", "Asia/Jakarta").expect("valid");
        // 10:30 UTC = 17:30 WIB; next is 18:00 WIB = 11:00 UTC.
        let after = Utc.with_ymd_and_hms(2026, 11, 12, 10, 30, 0).unwrap();
        assert_eq!(
            s.next_after(after),
            Some(Utc.with_ymd_and_hms(2026, 11, 12, 11, 0, 0).unwrap())
        );
        assert_eq!(s.describe(), "0 */6 * * * Asia/Jakarta");

        // SIMAN: Sunday 04:00 WIB. Thursday 2026-11-12 -> Sunday 2026-11-15 04:00 WIB = Saturday
        // 21:00 UTC.
        let s = Schedule::parse("0 4 * * 0", "Asia/Jakarta").expect("valid");
        assert_eq!(
            s.next_after(after),
            Some(Utc.with_ymd_and_hms(2026, 11, 14, 21, 0, 0).unwrap())
        );
    }

    #[test]
    fn a_schedule_that_cannot_be_understood_is_refused_not_guessed() {
        assert!(Schedule::parse("every six hours", "Asia/Jakarta").is_err());
        assert!(Schedule::parse("0 */6 * * *", "Asia/Djakarta").is_err());
    }

    #[test]
    fn a_long_reason_is_cut_on_a_character_boundary() {
        let long = "é".repeat(MAX_ERROR_CHARS + 50);
        let cut = shorten(&long);
        assert_eq!(cut.chars().count(), MAX_ERROR_CHARS + 1); // + the ellipsis
        assert_eq!(shorten("pendek"), "pendek");
    }
}
