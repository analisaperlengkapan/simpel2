//! Shared SLA time-unit helpers used by the step editor and SLA editor.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    Minutes,
    Hours,
    Days,
}

impl TimeUnit {
    pub fn code(self) -> &'static str {
        match self {
            TimeUnit::Minutes => "minutes",
            TimeUnit::Hours => "hours",
            TimeUnit::Days => "days",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            TimeUnit::Minutes => "Menit",
            TimeUnit::Hours => "Jam",
            TimeUnit::Days => "Hari",
        }
    }

    pub fn parse(code: &str) -> Self {
        match code {
            "hours" => TimeUnit::Hours,
            "days" => TimeUnit::Days,
            _ => TimeUnit::Minutes,
        }
    }

    pub fn to_minutes(self, value: u32) -> u32 {
        match self {
            TimeUnit::Minutes => value,
            TimeUnit::Hours => value.saturating_mul(60),
            TimeUnit::Days => value.saturating_mul(1440),
        }
    }
}

/// Pick the largest unit that divides evenly so the stored minutes show up
/// in the most natural form (e.g. 1440 → (1, Days), 90 → (90, Minutes)).
pub fn best_time_unit(minutes: u32) -> (u32, TimeUnit) {
    if minutes == 0 {
        return (0, TimeUnit::Minutes);
    }
    if minutes.is_multiple_of(1440) {
        (minutes / 1440, TimeUnit::Days)
    } else if minutes.is_multiple_of(60) {
        (minutes / 60, TimeUnit::Hours)
    } else {
        (minutes, TimeUnit::Minutes)
    }
}
