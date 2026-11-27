//! Progress of a pakaian dinas campaign ("kegiatan") for monitoring: how far each
//! level has got, from the whole campaign down to the satker.
//!
//! Three levels, each a fact the level above summarises:
//!
//! * the campaign ([`ProgresRingkasan`]): satker per workflow stage and how many
//!   employees have given a size;
//! * the wilayah ([`ProgresWilayah`]): the same figures per Kejati, laggards first,
//!   so the people who supervise a wilayah can see which one is behind;
//! * the satker ([`ProgresSatker`]): the ones furthest behind, the short list
//!   somebody has to chase. The employees of a satker are read from the roster
//!   page that already exists; the monitoring only points there.
//!
//! "Filled" is the same definition the readiness warnings use (a person the campaign
//! still expects a size from and who has at least one size saved; excluded people
//! owe nothing), so a number here never disagrees with a dialog.
//!
//! The grouping is pure ([`susun_progres`]) so it is testable without a database.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// How many satker the "furthest behind" list holds. A list somebody can act on,
/// not a second copy of the satker page.
pub const JUMLAH_TERTINGGAL: usize = 10;

/// Where a satker row is in the workflow, as monitoring groups it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TahapProgres {
    /// Still being filled, or sent back to the satker (1000/1003).
    BelumMengajukan,
    /// Submitted, waiting for the wilayah validator (1001/1007/1012).
    DiWilayah,
    /// Forwarded, waiting for pusat (1004/1010).
    DiPusat,
    /// Finished (1008).
    Selesai,
}

/// The monitoring stage of a status code. A code the workflow cannot produce counts
/// as "not submitted", the conservative reading: it never makes a campaign look
/// further along than it is.
pub fn tahap_progres(aktivitas_id: i32) -> TahapProgres {
    match aktivitas_id {
        1001 | 1007 | 1012 => TahapProgres::DiWilayah,
        1004 | 1010 => TahapProgres::DiPusat,
        1008 => TahapProgres::Selesai,
        _ => TahapProgres::BelumMengajukan,
    }
}

/// Figures shared by the campaign and by each wilayah.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgresRingkasan {
    pub satker_total: i64,
    /// Satker with nobody in the roster: nothing to fill in, nothing to submit.
    pub tanpa_pegawai: i64,
    pub belum_mengajukan: i64,
    pub di_wilayah: i64,
    pub di_pusat: i64,
    pub selesai: i64,
    pub pegawai_total: i64,
    pub pegawai_terisi: i64,
}

impl ProgresRingkasan {
    fn tambah(&mut self, baris: &BarisProgres) {
        self.satker_total += 1;
        if baris.pegawai_total == 0 {
            self.tanpa_pegawai += 1;
        }
        match tahap_progres(baris.aktivitas_id) {
            TahapProgres::BelumMengajukan => self.belum_mengajukan += 1,
            TahapProgres::DiWilayah => self.di_wilayah += 1,
            TahapProgres::DiPusat => self.di_pusat += 1,
            TahapProgres::Selesai => self.selesai += 1,
        }
        self.pegawai_total += baris.pegawai_total;
        self.pegawai_terisi += baris.pegawai_terisi;
    }

    /// Share of employees who have given a size, 0-100, or `None` when the roster is
    /// empty (there is nothing to be a share of).
    pub fn persen_terisi(&self) -> Option<i64> {
        persen(self.pegawai_terisi, self.pegawai_total)
    }
}

/// Whole-number percentage, or `None` for an empty denominator.
pub fn persen(bagian: i64, total: i64) -> Option<i64> {
    (total > 0).then(|| (bagian * 100) / total)
}

/// One wilayah (Kejati). `wilayah_code` is `None` for the satker that belong to no
/// Kejati (Kejaksaan Agung and its central units).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgresWilayah {
    pub wilayah_code: Option<String>,
    pub wilayah_nama: Option<String>,
    #[serde(flatten)]
    pub ringkasan: ProgresRingkasan,
}

/// One satker row for the "furthest behind" list.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgresSatker {
    /// The participation row, so a link can open it.
    pub id: String,
    pub satker_id: String,
    pub satker_nama: Option<String>,
    pub wilayah_nama: Option<String>,
    pub aktivitas_id: i32,
    pub pegawai_total: i64,
    pub pegawai_terisi: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgresPakaianDinas {
    pub ringkasan: ProgresRingkasan,
    pub per_wilayah: Vec<ProgresWilayah>,
    pub tertinggal: Vec<ProgresSatker>,
}

/// One satker row as the query returns it.
#[derive(Debug, Clone)]
pub struct BarisProgres {
    pub id: String,
    pub satker_id: String,
    pub satker_nama: Option<String>,
    pub wilayah_code: Option<String>,
    pub wilayah_nama: Option<String>,
    pub aktivitas_id: i32,
    pub pegawai_total: i64,
    pub pegawai_terisi: i64,
}

/// Group the satker rows into the campaign summary, the wilayah table and the
/// "furthest behind" list.
///
/// * Wilayah are ordered with the least-filled first (an empty roster has no rate and
///   goes last, by name), because the table exists to show who is behind.
/// * "Furthest behind" are the satker that are NOT finished, have people to fill in,
///   and have the lowest share filled; ties go to the one missing most people, then
///   by name so the order is stable.
pub fn susun_progres(baris: Vec<BarisProgres>) -> ProgresPakaianDinas {
    let mut ringkasan = ProgresRingkasan::default();
    let mut per: BTreeMap<(Option<String>, Option<String>), ProgresRingkasan> = BTreeMap::new();
    for b in &baris {
        ringkasan.tambah(b);
        per.entry((b.wilayah_code.clone(), b.wilayah_nama.clone()))
            .or_default()
            .tambah(b);
    }

    let mut per_wilayah: Vec<ProgresWilayah> = per
        .into_iter()
        .map(|((wilayah_code, wilayah_nama), ringkasan)| ProgresWilayah {
            wilayah_code,
            wilayah_nama,
            ringkasan,
        })
        .collect();
    per_wilayah.sort_by(|a, b| {
        // Least filled first; no rate (nobody to fill in) last; then by name.
        let rate = |w: &ProgresWilayah| w.ringkasan.persen_terisi();
        match (rate(a), rate(b)) {
            (Some(x), Some(y)) => x.cmp(&y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| a.wilayah_nama.cmp(&b.wilayah_nama))
    });

    let mut tertinggal: Vec<&BarisProgres> = baris
        .iter()
        .filter(|b| tahap_progres(b.aktivitas_id) != TahapProgres::Selesai && b.pegawai_total > 0)
        .collect();
    tertinggal.sort_by(|a, b| {
        let (ra, rb) = (
            a.pegawai_terisi * 1000 / a.pegawai_total,
            b.pegawai_terisi * 1000 / b.pegawai_total,
        );
        ra.cmp(&rb)
            .then_with(|| {
                let (ka, kb) = (
                    a.pegawai_total - a.pegawai_terisi,
                    b.pegawai_total - b.pegawai_terisi,
                );
                kb.cmp(&ka)
            })
            .then_with(|| a.satker_nama.cmp(&b.satker_nama))
            .then_with(|| a.satker_id.cmp(&b.satker_id))
    });
    let tertinggal = tertinggal
        .into_iter()
        .take(JUMLAH_TERTINGGAL)
        .map(|b| ProgresSatker {
            id: b.id.clone(),
            satker_id: b.satker_id.clone(),
            satker_nama: b.satker_nama.clone(),
            wilayah_nama: b.wilayah_nama.clone(),
            aktivitas_id: b.aktivitas_id,
            pegawai_total: b.pegawai_total,
            pegawai_terisi: b.pegawai_terisi,
        })
        .collect();

    ProgresPakaianDinas {
        ringkasan,
        per_wilayah,
        tertinggal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baris(
        id: &str,
        wilayah: Option<&str>,
        status: i32,
        total: i64,
        terisi: i64,
    ) -> BarisProgres {
        BarisProgres {
            id: id.to_string(),
            satker_id: id.to_string(),
            satker_nama: Some(format!("Satker {id}")),
            wilayah_code: wilayah.map(|w| w.to_string()),
            wilayah_nama: wilayah.map(|w| format!("Kejati {w}")),
            aktivitas_id: status,
            pegawai_total: total,
            pegawai_terisi: terisi,
        }
    }

    #[test]
    fn every_status_the_workflow_produces_has_a_stage() {
        for (code, tahap) in [
            (1000, TahapProgres::BelumMengajukan),
            (1003, TahapProgres::BelumMengajukan),
            (1001, TahapProgres::DiWilayah),
            (1007, TahapProgres::DiWilayah),
            (1012, TahapProgres::DiWilayah),
            (1004, TahapProgres::DiPusat),
            (1010, TahapProgres::DiPusat),
            (1008, TahapProgres::Selesai),
        ] {
            assert_eq!(tahap_progres(code), tahap, "{code}");
        }
        // A code the workflow cannot produce never makes the campaign look further along.
        assert_eq!(tahap_progres(1006), TahapProgres::BelumMengajukan);
        assert_eq!(tahap_progres(0), TahapProgres::BelumMengajukan);
    }

    #[test]
    fn the_summary_counts_satker_per_stage_and_employees_filled() {
        let p = susun_progres(vec![
            baris("a", Some("A"), 1000, 10, 2),
            baris("b", Some("A"), 1001, 4, 4),
            baris("c", Some("B"), 1004, 6, 6),
            baris("d", Some("B"), 1008, 5, 5),
            baris("e", Some("B"), 1000, 0, 0),
        ]);
        let r = &p.ringkasan;
        assert_eq!(
            (
                r.satker_total,
                r.belum_mengajukan,
                r.di_wilayah,
                r.di_pusat,
                r.selesai
            ),
            (5, 2, 1, 1, 1)
        );
        assert_eq!(r.tanpa_pegawai, 1);
        assert_eq!((r.pegawai_total, r.pegawai_terisi), (25, 17));
        assert_eq!(r.persen_terisi(), Some(68));
    }

    #[test]
    fn an_empty_roster_has_no_rate_rather_than_zero_or_a_division_by_zero() {
        assert_eq!(persen(0, 0), None);
        assert_eq!(persen(0, 5), Some(0));
        assert_eq!(ProgresRingkasan::default().persen_terisi(), None);
    }

    #[test]
    fn wilayah_come_least_filled_first_and_those_without_a_rate_last() {
        let p = susun_progres(vec![
            baris("a", Some("Penuh"), 1004, 10, 10),
            baris("b", Some("Tertinggal"), 1000, 10, 1),
            baris("c", Some("Sedang"), 1000, 10, 5),
            baris("d", Some("Kosong"), 1000, 0, 0),
        ]);
        let urutan: Vec<_> = p
            .per_wilayah
            .iter()
            .map(|w| w.wilayah_code.clone().unwrap())
            .collect();
        assert_eq!(urutan, ["Tertinggal", "Sedang", "Penuh", "Kosong"]);
    }

    #[test]
    fn satker_outside_any_wilayah_form_their_own_group() {
        let p = susun_progres(vec![
            baris("a", None, 1000, 2, 0),
            baris("b", Some("A"), 1000, 2, 2),
        ]);
        assert_eq!(p.per_wilayah.len(), 2);
        assert!(p.per_wilayah.iter().any(|w| w.wilayah_code.is_none()));
    }

    #[test]
    fn the_furthest_behind_skip_finished_and_empty_satker_and_lead_with_the_least_filled() {
        let p = susun_progres(vec![
            baris("selesai", Some("A"), 1008, 10, 0),
            baris("kosong", Some("A"), 1000, 0, 0),
            baris("hampir", Some("A"), 1000, 10, 9),
            baris("nol-kecil", Some("A"), 1000, 2, 0),
            baris("nol-besar", Some("A"), 1000, 20, 0),
            baris("separuh", Some("A"), 1001, 10, 5),
        ]);
        let ids: Vec<_> = p.tertinggal.iter().map(|s| s.id.as_str()).collect();
        // 0 %: the one missing most people first; then 50 %; then 90 %.
        assert_eq!(ids, ["nol-besar", "nol-kecil", "separuh", "hampir"]);
    }

    #[test]
    fn the_furthest_behind_list_is_bounded() {
        let banyak: Vec<_> = (0..40)
            .map(|i| baris(&format!("s{i:02}"), Some("A"), 1000, 10, 0))
            .collect();
        assert_eq!(susun_progres(banyak).tertinggal.len(), JUMLAH_TERTINGGAL);
    }
}
