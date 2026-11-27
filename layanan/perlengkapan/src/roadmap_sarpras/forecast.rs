//! Forecasting math over a yearly series. Pure: no I/O, no clock, so every number it produces can
//! be checked by a test that knows the answer.
//!
//! What the three methods are, stated once so the names mean what they say:
//!
//! * **SMA**: the mean of the last [`JENDELA`] years, projected flat.
//! * **WMA**: the same window with linearly increasing weights (the latest year weighs most),
//!   projected flat.
//! * **Holt** (`ExponentialSmoothing`): level and trend smoothing, with the two smoothing
//!   parameters FITTED to the series (the smallest one-step-ahead squared error over a grid)
//!   instead of fixed at 0.3 and 0.1, projected as `level + h * trend`.
//!
//! The uncertainty of each is the spread of that method's own one-step-ahead errors on the
//! history, never the spread of the raw series: a series that trends has a large raw spread
//! that says nothing about how well the trend is followed. A one-step error of a moving average
//! already contains the uncertainty of the window mean, so it is used as it is; only when there
//! is no such error yet (too few years) is the spread of the series used, and THEN the window
//! mean's own uncertainty is added. Holt's first prediction is fitted to the data that starts
//! it (the initial trend is `y1 - y0`), so that error is zero by construction and is left out.
//! For Holt the interval widens with the horizon by the exact variance of the method; the
//! moving averages assume the series stays where it is, so theirs does not widen. Where no
//! error can be measured at all, the notes say so ([`jumlah_galat`]).

use lib_perlengkapan::models::{ForecastMethod, PredictedYear, YearlyData};

/// Years the moving averages look at.
pub const JENDELA: usize = 3;

/// Fewer years than this and the interval is a rough guess, whatever the arithmetic says.
pub const MIN_TAHUN_YAKIN: usize = 5;

/// The projection of one series: the central value and the one-step error scale per horizon step
/// (`1..=horizon`).
#[derive(Debug, Clone, PartialEq)]
pub struct Proyeksi {
    pub titik: Vec<f64>,
    pub se: Vec<f64>,
}

// ── Normal quantile ──────────────────────────────────────────────────────────────────────────

/// Inverse of the standard normal CDF (Acklam's rational approximation, relative error below
/// 1.2e-9). The old table gave 1.96 for every confidence from 0.95 up to 0.98 and 1.0 for
/// everything below 0.80, so asking for 0.97 or 0.6 got another level's interval.
pub fn inv_norm(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969683028665376e+01,
        2.209460984245205e+02,
        -2.759285104469687e+02,
        1.383_577_518_672_69e2,
        -3.066479806614716e+01,
        2.506628277459239e+00,
    ];
    const B: [f64; 5] = [
        -5.447609879822406e+01,
        1.615858368580409e+02,
        -1.556989798598866e+02,
        6.680131188771972e+01,
        -1.328068155288572e+01,
    ];
    const C: [f64; 6] = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e+00,
        -2.549732539343734e+00,
        4.374664141464968e+00,
        2.938163982698783e+00,
    ];
    const D: [f64; 4] = [
        7.784695709041462e-03,
        3.224671290700398e-01,
        2.445134137142996e+00,
        3.754408661907416e+00,
    ];
    const P_LOW: f64 = 0.02425;

    let p = p.clamp(1e-12, 1.0 - 1e-12);
    if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= 1.0 - P_LOW {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    }
}

/// The z such that a normal variable lies within `±z` standard deviations with probability
/// `confidence`.
pub fn z_dua_sisi(confidence: f64) -> f64 {
    inv_norm(0.5 + confidence.clamp(0.5, 0.999) / 2.0)
}

// ── The series ───────────────────────────────────────────────────────────────────────────────

/// The most recent run of CONSECUTIVE years, and a note when older years were left out.
///
/// Every method here treats the series as one value per year in a row. A year with no rows is
/// absent from the aggregate, so "2018, 2019, 2021, 2022" read as four neighbours would put 2021
/// right after 2019 and tilt every trend. The years before the gap are set aside, and the note
/// says which.
pub fn deret_berurutan(historical: &[YearlyData]) -> (&[YearlyData], Option<String>) {
    let Some(last) = historical.len().checked_sub(1) else {
        return (historical, None);
    };
    let mut start = last;
    while start > 0 && historical[start - 1].tahun + 1 == historical[start].tahun {
        start -= 1;
    }
    let note = (start > 0).then(|| {
        format!(
            "Tahun {} sampai {} tidak dipakai: ada tahun tanpa data sebelum {}. Hanya {} sampai {} yang berurutan.",
            historical[0].tahun,
            historical[start - 1].tahun,
            historical[start].tahun,
            historical[start].tahun,
            historical[last].tahun,
        )
    });
    (&historical[start..], note)
}

fn rata(v: &[f64]) -> f64 {
    if v.is_empty() {
        0.0
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

/// Weighted mean with weights 1, 2, .. over the slice (the last one weighs most).
fn rata_berbobot(v: &[f64]) -> f64 {
    let total: f64 = (1..=v.len()).map(|w| w as f64).sum();
    if total == 0.0 {
        return 0.0;
    }
    v.iter()
        .enumerate()
        .map(|(i, x)| (i + 1) as f64 * x)
        .sum::<f64>()
        / total
}

fn simpangan_baku(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    let m = rata(v);
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
}

/// Root mean square of the one-step errors of a moving average over `y`: each year predicted by
/// the average of the `k` years before it, and whether that was measurable. With no year that has
/// `k` before it, the spread of the series itself is the only scale there is (`false`).
fn galat_rata_bergerak(y: &[f64], k: usize, bobot: bool) -> (f64, bool) {
    let mut kuadrat = 0.0;
    let mut m = 0usize;
    for t in k..y.len() {
        let window = &y[t - k..t];
        let pred = if bobot {
            rata_berbobot(window)
        } else {
            rata(window)
        };
        kuadrat += (y[t] - pred).powi(2);
        m += 1;
    }
    if m == 0 {
        (simpangan_baku(y), false)
    } else {
        ((kuadrat / m as f64).sqrt(), true)
    }
}

/// How many one-step errors `metode` can measure on `n` years: the moving averages need `k`
/// years before the first prediction, and Holt leaves out its first prediction (fitted to the
/// data that starts it) and so needs three years.
pub fn jumlah_galat(metode: &ForecastMethod, n: usize) -> usize {
    match metode {
        ForecastMethod::ExponentialSmoothing if n >= 2 => n - 2,
        _ => n.saturating_sub(n.clamp(1, JENDELA)),
    }
}

/// Sum of squared weights of a `k`-year window: the variance a window mean adds to one new
/// observation is `sigma^2 * (1 + this)`.
fn jumlah_bobot_kuadrat(k: usize, bobot: bool) -> f64 {
    if bobot {
        let atas: f64 = (1..=k).map(|i| (i * i) as f64).sum();
        let bawah: f64 = (1..=k).map(|i| i as f64).sum();
        atas / (bawah * bawah)
    } else {
        1.0 / k as f64
    }
}

/// One Holt pass: the final level and trend, and the sum of squared one-step errors (the first
/// prediction left out, see below).
fn holt(y: &[f64], alpha: f64, beta: f64) -> (f64, f64, f64) {
    let mut level = y[0];
    let mut trend = y[1] - y[0];
    let mut sse = 0.0;
    for v in &y[1..] {
        let pred = level + trend;
        // The prediction of the second year uses an initial trend taken FROM the second year, so
        // its error is exactly zero: it adds nothing to the sum, but it is not a measurement
        // either, and `jumlah_galat` does not count it (without that, two years looked like one
        // measured error of zero and the band collapsed to a point).
        sse += (v - pred).powi(2);
        let new_level = alpha * v + (1.0 - alpha) * pred;
        trend = beta * (new_level - level) + (1.0 - beta) * trend;
        level = new_level;
    }
    (level, trend, sse)
}

/// Project one series `horizon` years past its last year. `y` is oldest first, one value per
/// consecutive year, and must hold at least one value (two for Holt; with one it falls back to
/// the moving average).
pub fn proyeksi(metode: &ForecastMethod, y: &[f64], horizon: usize) -> Proyeksi {
    let n = y.len();
    if n == 0 {
        return Proyeksi {
            titik: vec![0.0; horizon],
            se: vec![0.0; horizon],
        };
    }
    let k = n.clamp(1, JENDELA);
    match metode {
        ForecastMethod::Sma | ForecastMethod::Wma => {
            let bobot = matches!(metode, ForecastMethod::Wma);
            let window = &y[n - k..];
            let level = if bobot {
                rata_berbobot(window)
            } else {
                rata(window)
            };
            let (sigma, terukur) = galat_rata_bergerak(y, k, bobot);
            // A measured one-step error already contains the uncertainty of the window mean; the
            // spread of the series is only the spread of observations around THEIR mean, so one new
            // observation around a mean estimated from `k` of them adds the mean's own variance.
            let se = if terukur {
                sigma
            } else {
                sigma * (1.0 + jumlah_bobot_kuadrat(k, bobot)).sqrt()
            };
            Proyeksi {
                titik: vec![level; horizon],
                se: vec![se; horizon],
            }
        }
        ForecastMethod::ExponentialSmoothing if n >= 2 => {
            let mut terbaik = (f64::INFINITY, 0.0, 0.0, 0.0, 0.0);
            for a in 1..=9 {
                let alpha = a as f64 / 10.0;
                for beta in [0.05, 0.1, 0.2, 0.3, 0.5] {
                    let (level, trend, sse) = holt(y, alpha, beta);
                    if sse < terbaik.0 {
                        terbaik = (sse, level, trend, alpha, beta);
                    }
                }
            }
            let (sse, level, trend, alpha, beta) = terbaik;
            let galat = jumlah_galat(metode, n);
            // Two parameters were fitted: take them off the degrees of freedom, as long as any
            // remain. With no measurable error (two years) the spread of the series is the only
            // scale there is.
            let sigma = if galat == 0 {
                simpangan_baku(y)
            } else {
                (sse / galat.saturating_sub(2).max(1) as f64).sqrt()
            };
            let mut titik = Vec::with_capacity(horizon);
            let mut se = Vec::with_capacity(horizon);
            for h in 1..=horizon {
                titik.push(level + h as f64 * trend);
                // Exact variance of Holt's method: 1 + sum_{j<h} (alpha (1 + j beta))^2.
                let tambahan: f64 = (1..h)
                    .map(|j| (alpha * (1.0 + j as f64 * beta)).powi(2))
                    .sum();
                se.push(sigma * (1.0 + tambahan).sqrt());
            }
            Proyeksi { titik, se }
        }
        // Holt needs two points; one point has nothing to smooth.
        _ => proyeksi(&ForecastMethod::Sma, y, horizon),
    }
}

/// The predicted years for a series of yearly rows: kebutuhan with its interval, and gap and cost
/// projected the same way (so one method describes all three columns).
pub fn prediksi(
    metode: &ForecastMethod,
    deret: &[YearlyData],
    horizon: usize,
    confidence: f64,
) -> Vec<PredictedYear> {
    if deret.is_empty() {
        return Vec::new();
    }
    let kebutuhan: Vec<f64> = deret.iter().map(|h| h.total_kebutuhan as f64).collect();
    let gap: Vec<f64> = deret.iter().map(|h| h.total_gap as f64).collect();
    let biaya: Vec<f64> = deret.iter().map(|h| h.estimasi_total_biaya).collect();
    let pk = proyeksi(metode, &kebutuhan, horizon);
    let pg = proyeksi(metode, &gap, horizon);
    let pb = proyeksi(metode, &biaya, horizon);
    let z = z_dua_sisi(confidence);
    let tahun_akhir = deret.last().map(|h| h.tahun).unwrap_or_default();
    (0..horizon)
        .map(|i| {
            let tengah = pk.titik[i].max(0.0);
            let margin = z * pk.se[i];
            PredictedYear {
                tahun: tahun_akhir + i as i32 + 1,
                predicted_kebutuhan: tengah,
                // The gap is the part of the need that is not met: never negative, never above the need
                // (two trends fitted separately can cross).
                predicted_gap: pg.titik[i].max(0.0).min(tengah),
                predicted_biaya: pb.titik[i].max(0.0),
                confidence_lower: (pk.titik[i] - margin).max(0.0),
                confidence_upper: (pk.titik[i] + margin).max(tengah),
            }
        })
        .collect()
}

/// Compound yearly growth of kebutuhan over the series, in percent. The old figure was the plain
/// mean of the year-on-year percentages, which a small base year inflates (2 to 6 is +200% and
/// outweighs five years of +1%); compound growth is what the first and last year imply.
/// `None` when it is not defined (fewer than two years, or a first year of zero).
pub fn pertumbuhan_majemuk_persen(deret: &[YearlyData]) -> Option<f64> {
    let (awal, akhir) = (deret.first()?, deret.last()?);
    let selisih = deret.len().checked_sub(1).filter(|n| *n >= 1)?;
    if awal.total_kebutuhan <= 0 || akhir.total_kebutuhan < 0 {
        return None;
    }
    let rasio = akhir.total_kebutuhan as f64 / awal.total_kebutuhan as f64;
    Some((rasio.powf(1.0 / selisih as f64) - 1.0) * 100.0)
}

/// What to tell the reader about how far to trust the result.
pub fn catatan(n_tahun: usize, metode: &ForecastMethod, gap: Option<String>) -> Vec<String> {
    let mut out: Vec<String> = gap.into_iter().collect();
    if jumlah_galat(metode, n_tahun) == 0 {
        out.push(format!(
            "Dengan {n_tahun} tahun data galat prediksi belum bisa diukur: rentang memakai simpangan baku deret dan hanya perkiraan kasar."
        ));
    }
    if n_tahun < MIN_TAHUN_YAKIN {
        out.push(format!(
            "Hanya {n_tahun} tahun data: rentang kepercayaan hanya perkiraan kasar (disarankan sedikitnya {MIN_TAHUN_YAKIN} tahun)."
        ));
    }
    if matches!(metode, ForecastMethod::Sma | ForecastMethod::Wma) {
        out.push(format!(
            "Rata-rata bergerak ({JENDELA} tahun terakhir) memproyeksikan nilai datar: tidak mengikuti tren."
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tahunan(mulai: i32, nilai: &[i64]) -> Vec<YearlyData> {
        nilai
            .iter()
            .enumerate()
            .map(|(i, v)| YearlyData {
                tahun: mulai + i as i32,
                total_kebutuhan: *v,
                total_existing: 0,
                total_gap: *v / 2,
                jumlah_satker: 1,
                estimasi_total_biaya: *v as f64 * 1000.0,
            })
            .collect()
    }

    #[test]
    fn the_normal_quantile_is_the_real_one_at_every_confidence() {
        // Textbook two-sided values. The old step table answered 1.96 for 0.97 and 1.0 for 0.6.
        for (c, z) in [
            (0.5, 0.67449),
            (0.6, 0.84162),
            (0.8, 1.28155),
            (0.9, 1.64485),
            (0.95, 1.95996),
            (0.97, 2.17009),
            (0.99, 2.57583),
        ] {
            let got = z_dua_sisi(c);
            assert!((got - z).abs() < 1e-4, "confidence {c}: {got} vs {z}");
        }
        assert!(inv_norm(0.5).abs() < 1e-9);
        assert!(
            (inv_norm(0.975) + inv_norm(0.025)).abs() < 1e-9,
            "symmetric"
        );
    }

    #[test]
    fn a_gap_in_the_years_sets_the_older_ones_aside_and_says_so() {
        // 2018 to 2020, no 2021, then 2022 to 2024.
        let mut h = tahunan(2018, &[10, 11, 12]);
        h.extend(tahunan(2022, &[13, 14, 15]));
        let (deret, catatan) = deret_berurutan(&h);
        assert_eq!(deret.first().unwrap().tahun, 2022);
        assert_eq!(deret.len(), 3);
        let c = catatan.expect("a note");
        assert!(
            c.contains("2018") && c.contains("2020") && c.contains("2022") && c.contains("2024"),
            "{c}"
        );

        let tidak_bolong = tahunan(2020, &[1, 2, 3, 4]);
        let (deret, catatan) = deret_berurutan(&tidak_bolong);
        assert_eq!(deret.len(), 4);
        assert!(catatan.is_none());
        assert!(deret_berurutan(&[]).0.is_empty());
    }

    #[test]
    fn sma_is_the_mean_of_the_last_window_not_of_every_year() {
        let y = [1.0, 1.0, 1.0, 10.0, 20.0, 30.0];
        let p = proyeksi(&ForecastMethod::Sma, &y, 3);
        assert_eq!(p.titik, vec![20.0, 20.0, 20.0]);
    }

    #[test]
    fn wma_weights_the_latest_year_most() {
        // Weights 1, 2, 3 over 10, 20, 30: (10 + 40 + 90) / 6.
        let p = proyeksi(&ForecastMethod::Wma, &[10.0, 20.0, 30.0], 1);
        assert!((p.titik[0] - 140.0 / 6.0).abs() < 1e-9, "{:?}", p.titik);
        let sma = proyeksi(&ForecastMethod::Sma, &[10.0, 20.0, 30.0], 1);
        assert!(p.titik[0] > sma.titik[0]);
    }

    #[test]
    fn holt_follows_a_straight_line_and_is_sure_about_it() {
        let y = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0];
        let p = proyeksi(&ForecastMethod::ExponentialSmoothing, &y, 3);
        for (i, v) in p.titik.iter().enumerate() {
            let benar = 70.0 + 10.0 * i as f64;
            assert!((v - benar).abs() < 1e-6, "year +{}: {v} vs {benar}", i + 1);
        }
        assert!(
            p.se.iter().all(|s| *s < 1e-6),
            "no error to speak of: {:?}",
            p.se
        );
    }

    #[test]
    fn holt_interval_widens_with_the_horizon_when_the_series_is_noisy() {
        let y = [10.0, 14.0, 9.0, 15.0, 11.0, 16.0, 12.0];
        let p = proyeksi(&ForecastMethod::ExponentialSmoothing, &y, 5);
        assert!(p.se.windows(2).all(|w| w[1] >= w[0]), "{:?}", p.se);
        assert!(p.se[4] > p.se[0]);
    }

    #[test]
    fn a_constant_series_projects_flat_with_no_width_for_every_method() {
        for m in [
            ForecastMethod::Sma,
            ForecastMethod::Wma,
            ForecastMethod::ExponentialSmoothing,
        ] {
            let p = proyeksi(&m, &[7.0; 6], 4);
            assert!(p.titik.iter().all(|v| (v - 7.0).abs() < 1e-9), "{m:?}");
            assert!(p.se.iter().all(|s| s.abs() < 1e-9), "{m:?}");
        }
    }

    #[test]
    fn the_spread_is_the_methods_own_error_not_the_raw_series() {
        // A clean upward line has a large raw spread and no error for Holt to speak of: the old
        // interval used the raw spread and was wide on a series the method follows perfectly.
        let y = [100.0, 200.0, 300.0, 400.0, 500.0];
        assert!(simpangan_baku(&y) > 100.0);
        let p = proyeksi(&ForecastMethod::ExponentialSmoothing, &y, 2);
        assert!(p.se[0] < 1e-6);
    }

    #[test]
    fn predicted_years_follow_the_last_year_and_the_band_brackets_the_value() {
        let h = tahunan(2020, &[10, 14, 9, 15, 11, 16]);
        for m in [
            ForecastMethod::Sma,
            ForecastMethod::Wma,
            ForecastMethod::ExponentialSmoothing,
        ] {
            let out = prediksi(&m, &h, 4, 0.95);
            assert_eq!(
                out.iter().map(|p| p.tahun).collect::<Vec<_>>(),
                vec![2026, 2027, 2028, 2029]
            );
            for p in &out {
                assert!(p.confidence_lower <= p.predicted_kebutuhan, "{m:?} {p:?}");
                assert!(p.predicted_kebutuhan <= p.confidence_upper, "{m:?} {p:?}");
                assert!(
                    p.confidence_lower >= 0.0 && p.predicted_gap >= 0.0 && p.predicted_biaya >= 0.0
                );
            }
        }
    }

    #[test]
    fn a_falling_series_never_forecasts_below_zero() {
        let h = tahunan(2020, &[50, 30, 12, 4, 1]);
        let out = prediksi(&ForecastMethod::ExponentialSmoothing, &h, 6, 0.95);
        assert!(
            out.iter()
                .all(|p| p.predicted_kebutuhan >= 0.0 && p.confidence_lower >= 0.0)
        );
    }

    #[test]
    fn a_higher_confidence_is_a_wider_band() {
        let h = tahunan(2018, &[10, 14, 9, 15, 11, 16, 12]);
        let a = prediksi(&ForecastMethod::Sma, &h, 1, 0.80);
        let b = prediksi(&ForecastMethod::Sma, &h, 1, 0.99);
        assert!(
            b[0].confidence_upper - b[0].confidence_lower
                > a[0].confidence_upper - a[0].confidence_lower
        );
    }

    #[test]
    fn growth_is_compound_not_the_mean_of_yearly_percentages() {
        // 100 -> 121 in two steps is 10% a year.
        let g = pertumbuhan_majemuk_persen(&tahunan(2020, &[100, 110, 121])).unwrap();
        assert!((g - 10.0).abs() < 1e-9, "{g}");
        // The old mean of yearly percentages for 2, 6, 6, 6, 6: (200 + 0 + 0 + 0) / 4 = 50% a year,
        // for a series that has been flat for four years; compound growth is the real (2 -> 6) rate.
        let g = pertumbuhan_majemuk_persen(&tahunan(2020, &[2, 6, 6, 6, 6])).unwrap();
        assert!(
            (g - ((6.0_f64 / 2.0).powf(0.25) - 1.0) * 100.0).abs() < 1e-9,
            "{g}"
        );
        // Undefined cases are None, not 0.
        assert!(pertumbuhan_majemuk_persen(&tahunan(2020, &[5])).is_none());
        assert!(pertumbuhan_majemuk_persen(&tahunan(2020, &[0, 5, 9])).is_none());
        assert!(pertumbuhan_majemuk_persen(&[]).is_none());
    }

    fn dengan_gap(kebutuhan: &[i64], gap: &[i64]) -> Vec<YearlyData> {
        kebutuhan
            .iter()
            .zip(gap)
            .enumerate()
            .map(|(i, (k, g))| YearlyData {
                tahun: 2020 + i as i32,
                total_kebutuhan: *k,
                total_existing: 0,
                total_gap: *g,
                jumlah_satker: 1,
                estimasi_total_biaya: *k as f64,
            })
            .collect()
    }

    #[test]
    fn a_projected_gap_never_exceeds_the_projected_need() {
        // Flat need, rising gap: Holt follows the gap's trend to 125 against a need of 100.
        let h = dengan_gap(&[100; 5], &[0, 25, 50, 75, 100]);
        for m in [
            ForecastMethod::Sma,
            ForecastMethod::Wma,
            ForecastMethod::ExponentialSmoothing,
        ] {
            for p in prediksi(&m, &h, 4, 0.95) {
                assert!(
                    p.predicted_gap <= p.predicted_kebutuhan + 1e-9,
                    "{m:?}: gap {} above need {}",
                    p.predicted_gap,
                    p.predicted_kebutuhan
                );
            }
        }
    }

    #[test]
    fn holt_on_two_years_does_not_report_a_zero_width_band() {
        // The second year is predicted with a trend taken FROM the second year: no error to measure.
        let p = proyeksi(&ForecastMethod::ExponentialSmoothing, &[100.0, 120.0], 1);
        assert!((p.titik[0] - 140.0).abs() < 1e-9);
        assert!(
            p.se[0] > 1.0,
            "a band that says 140 to 140 is a lie: {:?}",
            p.se
        );
        assert_eq!(jumlah_galat(&ForecastMethod::ExponentialSmoothing, 2), 0);
        assert!(
            catatan(2, &ForecastMethod::ExponentialSmoothing, None)
                .iter()
                .any(|n| n.contains("belum bisa diukur"))
        );
    }

    #[test]
    fn holt_leaves_its_first_prediction_out_of_the_error() {
        // Three years: one measurable error (the third year against the trend of the first two).
        assert_eq!(jumlah_galat(&ForecastMethod::ExponentialSmoothing, 3), 1);
        let p = proyeksi(
            &ForecastMethod::ExponentialSmoothing,
            &[10.0, 20.0, 35.0],
            1,
        );
        assert!(p.se[0] > 1.0, "{:?}", p.se);
    }

    #[test]
    fn a_measured_moving_average_error_is_not_inflated_a_second_time() {
        // 10, 20, 10, 20, 10, 20 predicted by the mean of the 3 before: errors +6.67, -6.67, +6.67,
        // so the one-step RMSE is 20/3, and that IS the scale of a new observation. (Multiplying it by
        // sqrt(1 + 1/3) counted the window mean's uncertainty twice.)
        let y = [10.0, 20.0, 10.0, 20.0, 10.0, 20.0];
        let p = proyeksi(&ForecastMethod::Sma, &y, 2);
        assert!((p.se[0] - 20.0 / 3.0).abs() < 1e-9, "{:?}", p.se);
        assert_eq!(jumlah_galat(&ForecastMethod::Sma, 6), 3);
    }

    #[test]
    fn without_a_measured_error_the_window_mean_uncertainty_is_added() {
        // Three years: no year has three before it. The spread of the series (10) stands for the
        // spread of observations; a new one around a mean of 3 adds 1/3 of a variance.
        let y = [10.0, 20.0, 30.0];
        let sma = proyeksi(&ForecastMethod::Sma, &y, 1);
        assert!((sma.se[0] - 10.0 * (1.0_f64 + 1.0 / 3.0).sqrt()).abs() < 1e-9);
        // WMA's weights are 1/6, 2/6, 3/6: the sum of their squares is 14/36.
        let wma = proyeksi(&ForecastMethod::Wma, &y, 1);
        assert!((wma.se[0] - 10.0 * (1.0_f64 + 14.0 / 36.0).sqrt()).abs() < 1e-9);
        assert_eq!(jumlah_galat(&ForecastMethod::Sma, 3), 0);
    }

    #[test]
    fn the_notes_warn_about_few_years_and_flat_projections() {
        let c = catatan(3, &ForecastMethod::Sma, None);
        assert!(c.iter().any(|n| n.contains("Hanya 3 tahun")));
        assert!(c.iter().any(|n| n.contains("datar")));
        let c = catatan(8, &ForecastMethod::ExponentialSmoothing, Some("x".into()));
        assert_eq!(c, vec!["x".to_string()]);
    }
}
