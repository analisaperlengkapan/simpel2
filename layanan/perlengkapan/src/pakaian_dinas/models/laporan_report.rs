//! # Shared report view-models for Pakaian Dinas laporan
//!
//! Both the Excel (`export.rs`) and PDF (`pdf_export.rs`) generators render
//! from these typed view-models, so the two output formats can never drift
//! apart in content again. The repository returns *raw long-format rows*
//! ([`DaftarLongRow`], [`RekapLongRow`]); the pure [`build_daftar_report`] /
//! [`build_rekap_report`] functions pivot them into the view-models and are
//! unit-tested without a database.
//!
//! Layout parity target = legacy simpelv1 `cetakDaftarTemplateV` /
//! `cetakRekapTemplateV`:
//! - **Daftar**: dynamic clothing columns (one per configured spesifikasi),
//!   grouped per satker, with a nama/periode/filter header.
//! - **Rekap**: one block per pakaian × gender, rows = satker, columns =
//!   ukuran, with a per-block summary row.

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// ───────────────────────── Shared header ─────────────────────────

/// Document header shared by both reports.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportHeader {
    /// Pengajuan name (legacy `$header->nama`).
    pub judul: String,
    /// Who the report covers (legacy `$header->inst_nama`): the satker's name
    /// for a single-satker report, otherwise `WILAYAH <name>`.
    pub inst_nama: String,
    pub periode_mulai: Option<NaiveDate>,
    pub periode_selesai: Option<NaiveDate>,
    /// Applied filters as (label, value) pairs (legacy `$filter`).
    pub filters: Vec<(String, String)>,
}

/// The legacy date format (`MyHelper::dateFormatIndo`): `dd-<Bulan>-yyyy`, with
/// the Indonesian month names as simpelv1 spells them (November is "Nopember").
/// A missing date is an empty string, as there.
pub fn tanggal_indo(d: Option<NaiveDate>) -> String {
    use chrono::Datelike;
    const BULAN: [&str; 12] = [
        "Januari",
        "Februari",
        "Maret",
        "April",
        "Mei",
        "Juni",
        "Juli",
        "Agustus",
        "September",
        "Oktober",
        "Nopember",
        "Desember",
    ];
    match d {
        Some(d) => format!("{:02}-{}-{}", d.day(), BULAN[d.month0() as usize], d.year()),
        None => String::new(),
    }
}

impl ReportHeader {
    /// The third heading line, exactly as the legacy templates print it.
    pub fn periode_line(&self) -> String {
        format!(
            "Periode: {} S/D {}",
            tanggal_indo(self.periode_mulai),
            tanggal_indo(self.periode_selesai)
        )
    }
}

/// Drops repeated names, keeping the first of each in its place.
fn dedup(names: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    names
        .into_iter()
        .filter(|n| seen.insert(n.clone()))
        .collect()
}

/// Map `jenis` code to the legacy Jaksa/TU status letter.
fn status_letter(jenis: Option<&str>) -> &'static str {
    match jenis {
        Some("0") => "J",
        Some("1") => "T",
        _ => "-",
    }
}

// ───────────────────────── Daftar ─────────────────────────

/// One raw row from the daftar query: a pegawai joined with (at most) one of
/// their clothing-item sizes. A pegawai with no recorded sizes still yields a
/// single row with `item_nama`/`ukuran` = `None`.
#[derive(Debug, Clone)]
pub struct DaftarLongRow {
    pub satker_nama: String,
    pub nip: String,
    pub nama: String,
    pub jabatan: Option<String>,
    pub golongan: Option<String>,
    /// "0" = Jaksa, "1" = TU.
    pub jenis: Option<String>,
    pub jenis_kelamin: String,
    pub with_hijab: bool,
    pub item_nama: Option<String>,
    pub ukuran: Option<String>,
}

/// A rendered pegawai row; `sizes` is aligned 1:1 with [`DaftarReport::columns`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaftarRow {
    pub nip: String,
    pub nama: String,
    pub jabatan: String,
    pub golongan: String,
    /// J / T / -
    pub status: String,
    pub gender: String,
    /// Y / T
    pub hijab: String,
    pub sizes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaftarSatkerGroup {
    pub satker_nama: String,
    pub rows: Vec<DaftarRow>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DaftarReport {
    pub header: ReportHeader,
    /// Dynamic clothing-item column labels (one per configured spesifikasi).
    pub columns: Vec<String>,
    pub satkers: Vec<DaftarSatkerGroup>,
}

struct DaftarPegawaiAccum {
    nip: String,
    nama: String,
    jabatan: String,
    golongan: String,
    status: String,
    gender: String,
    hijab: String,
    sizes: Vec<Option<String>>,
}

/// Pivot raw long rows into a per-satker, dynamic-column daftar report.
///
/// `columns` is the authoritative, ordered list of clothing-item labels. Sizes
/// whose `item_nama` is not among `columns` are ignored (defensive). Satker and
/// pegawai order follow first appearance in `rows` (the query orders by satker
/// then nama).
pub fn build_daftar_report(
    header: ReportHeader,
    columns: Vec<String>,
    satker_names: Vec<String>,
    rows: Vec<DaftarLongRow>,
) -> DaftarReport {
    let col_index: HashMap<&str, usize> = columns
        .iter()
        .enumerate()
        .map(|(i, c)| (c.as_str(), i))
        .collect();
    let n_cols = columns.len();

    // Every satker of the campaign gets its own block, in the given order, even
    // when nobody in it has been approved yet: the legacy report prints the
    // heading and a "Data Kosong" row for it. A satker that shows up in `rows`
    // but not in the list is appended rather than dropped.
    let mut satker_order: Vec<String> = dedup(satker_names);
    // satker_nama -> (nip order, nip -> accum)
    let mut satkers: HashMap<String, (Vec<String>, HashMap<String, DaftarPegawaiAccum>)> =
        satker_order
            .iter()
            .map(|n| (n.clone(), (Vec::new(), HashMap::new())))
            .collect();

    for row in rows {
        let satker_key = if row.satker_nama.is_empty() {
            "-".to_string()
        } else {
            row.satker_nama.clone()
        };
        let entry = satkers.entry(satker_key.clone()).or_insert_with(|| {
            satker_order.push(satker_key.clone());
            (Vec::new(), HashMap::new())
        });

        let pegawai = entry.1.entry(row.nip.clone()).or_insert_with(|| {
            entry.0.push(row.nip.clone());
            DaftarPegawaiAccum {
                nip: row.nip.clone(),
                nama: row.nama.clone(),
                jabatan: row.jabatan.clone().unwrap_or_else(|| "-".to_string()),
                golongan: row.golongan.clone().unwrap_or_else(|| "-".to_string()),
                status: status_letter(row.jenis.as_deref()).to_string(),
                gender: row.jenis_kelamin.clone(),
                hijab: if row.with_hijab { "Y" } else { "T" }.to_string(),
                sizes: vec![None; n_cols],
            }
        });

        if let (Some(item), Some(ukuran)) = (row.item_nama.as_deref(), row.ukuran)
            && let Some(&idx) = col_index.get(item)
        {
            pegawai.sizes[idx] = Some(ukuran);
        }
    }

    let groups = satker_order
        .into_iter()
        .map(|satker_nama| {
            let (nip_order, mut map) = satkers.remove(&satker_nama).expect("satker present");
            let rows = nip_order
                .into_iter()
                .map(|nip| {
                    let a = map.remove(&nip).expect("pegawai present");
                    DaftarRow {
                        nip: a.nip,
                        nama: a.nama,
                        jabatan: a.jabatan,
                        golongan: a.golongan,
                        status: a.status,
                        gender: a.gender,
                        hijab: a.hijab,
                        sizes: a
                            .sizes
                            .into_iter()
                            .map(|s| s.unwrap_or_else(|| "-".to_string()))
                            .collect(),
                    }
                })
                .collect();
            DaftarSatkerGroup { satker_nama, rows }
        })
        .collect();

    DaftarReport {
        header,
        columns,
        satkers: groups,
    }
}

// ───────────────────────── Rekap ─────────────────────────

/// One raw row from the rekap query: count of a (pakaian, gender, satker,
/// ukuran) combination.
#[derive(Debug, Clone)]
pub struct RekapLongRow {
    pub pakaian_nama: String,
    pub ukuran_group: String,
    /// "L" / "P"
    pub gender: String,
    pub satker_nama: String,
    pub ukuran: String,
    pub jumlah: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RekapSatkerRow {
    pub satker_nama: String,
    /// Aligned 1:1 with [`RekapBlock::ukuran_labels`].
    pub counts: Vec<i64>,
    pub total: i64,
}

/// One table = one pakaian × gender (legacy renders each on its own page).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RekapBlock {
    pub pakaian_nama: String,
    pub ukuran_group: String,
    /// "L" / "P"
    pub gender: String,
    pub ukuran_labels: Vec<String>,
    pub satkers: Vec<RekapSatkerRow>,
    /// Column totals, aligned with `ukuran_labels`.
    pub summary: Vec<i64>,
    pub summary_total: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RekapReport {
    pub header: ReportHeader,
    pub blocks: Vec<RekapBlock>,
}

/// One clothing column of the campaign as the legacy report reads it
/// (`pengajuan_pakaian_dinas_pakaian`): its name, the size group that fixes the
/// columns of its table, and which gender it is for.
#[derive(Debug, Clone)]
pub struct RekapPakaian {
    pub nama: String,
    pub ukuran_group: String,
    /// `subspesifikasi_gender`: "L" or "P" gives that one table, anything else
    /// (empty, "SEMUA") gives both.
    pub gender: String,
}

/// Pivot raw long rows into the legacy rekap: one block per (pakaian × gender),
/// EVERY size of the group as a column, EVERY satker of the campaign as a row
/// (zeros included), and a "Jumlah" row.
///
/// This is deliberately not "only what occurs". The recap used to drop the
/// sizes nobody picked and the satker nobody had finished, which made each page
/// a different shape; the legacy tables are fixed-shape so a reader can compare
/// pages and spot an empty satker at a glance.
///
/// `ukuran_order` maps an `ukuran_group` to its master-ordered label list (by
/// `urutan`). A size present in `rows` but absent from the master list is
/// appended (sorted) so no data is dropped; likewise a satker in `rows` that is
/// not in `satker_names`.
pub fn build_rekap_report(
    header: ReportHeader,
    ukuran_order: &HashMap<String, Vec<String>>,
    pakaians: Vec<RekapPakaian>,
    satker_names: Vec<String>,
    rows: Vec<RekapLongRow>,
) -> RekapReport {
    // (pakaian, gender, satker) -> ukuran -> count
    let mut counts: HashMap<(String, String, String), HashMap<String, i64>> = HashMap::new();
    let satker_names = dedup(satker_names);
    let mut extra_satkers: Vec<String> = Vec::new();
    for r in &rows {
        if !satker_names.contains(&r.satker_nama) && !extra_satkers.contains(&r.satker_nama) {
            extra_satkers.push(r.satker_nama.clone());
        }
        *counts
            .entry((
                r.pakaian_nama.clone(),
                r.gender.clone(),
                r.satker_nama.clone(),
            ))
            .or_default()
            .entry(r.ukuran.clone())
            .or_insert(0) += r.jumlah;
    }
    extra_satkers.sort();
    let all_satkers: Vec<String> = satker_names.into_iter().chain(extra_satkers).collect();

    let mut seen: HashSet<(String, &'static str)> = HashSet::new();
    let mut blocks = Vec::new();
    for p in pakaians {
        let genders: &[&'static str] = match p.gender.as_str() {
            "L" => &["L"],
            "P" => &["P"],
            _ => &["L", "P"],
        };
        for &gender in genders {
            // The same table is never printed twice (a spesifikasi with a
            // "SEMUA" row and a "L" sub-row would otherwise repeat the L page).
            if !seen.insert((p.nama.clone(), gender)) {
                continue;
            }
            let master: Vec<String> = ukuran_order
                .get(&p.ukuran_group)
                .cloned()
                .unwrap_or_default();
            let mut extras: Vec<String> = all_satkers
                .iter()
                .filter_map(|s| counts.get(&(p.nama.clone(), gender.to_string(), s.clone())))
                .flat_map(|m| m.keys().cloned())
                .filter(|u| !master.contains(u))
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            extras.sort();
            let labels: Vec<String> = master.into_iter().chain(extras).collect();

            let mut summary = vec![0i64; labels.len()];
            let satkers: Vec<RekapSatkerRow> = all_satkers
                .iter()
                .map(|satker| {
                    let by_size = counts.get(&(p.nama.clone(), gender.to_string(), satker.clone()));
                    let row_counts: Vec<i64> = labels
                        .iter()
                        .map(|l| by_size.and_then(|m| m.get(l)).copied().unwrap_or(0))
                        .collect();
                    for (i, c) in row_counts.iter().enumerate() {
                        summary[i] += c;
                    }
                    RekapSatkerRow {
                        satker_nama: satker.clone(),
                        total: row_counts.iter().sum(),
                        counts: row_counts,
                    }
                })
                .collect();
            let summary_total = summary.iter().sum();
            blocks.push(RekapBlock {
                pakaian_nama: p.nama.clone(),
                ukuran_group: p.ukuran_group.clone(),
                gender: gender.to_string(),
                ukuran_labels: labels,
                satkers,
                summary,
                summary_total,
            });
        }
    }
    RekapReport { header, blocks }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dlr(satker: &str, nip: &str, item: Option<&str>, ukuran: Option<&str>) -> DaftarLongRow {
        DaftarLongRow {
            satker_nama: satker.to_string(),
            nip: nip.to_string(),
            nama: format!("Nama {nip}"),
            jabatan: Some("Jaksa".to_string()),
            golongan: Some("III/a".to_string()),
            jenis: Some("0".to_string()),
            jenis_kelamin: "L".to_string(),
            with_hijab: false,
            item_nama: item.map(String::from),
            ukuran: ukuran.map(String::from),
        }
    }

    #[test]
    fn daftar_pivots_dynamic_columns_per_satker() {
        let columns = vec![
            "Kemeja".to_string(),
            "Sepatu".to_string(),
            "Topi".to_string(),
        ];
        let rows = vec![
            dlr("Kejari A", "1", Some("Kemeja"), Some("L")),
            dlr("Kejari A", "1", Some("Sepatu"), Some("42")),
            dlr("Kejari A", "1", Some("Topi"), Some("M")),
            // pegawai with no sizes at all
            dlr("Kejari A", "2", None, None),
            dlr("Kejari B", "3", Some("Topi"), Some("L")),
        ];
        let rep = build_daftar_report(
            ReportHeader::default(),
            columns,
            vec!["Kejari A".to_string(), "Kejari B".to_string()],
            rows,
        );

        assert_eq!(rep.satkers.len(), 2);
        let a = &rep.satkers[0];
        assert_eq!(a.satker_nama, "Kejari A");
        assert_eq!(a.rows.len(), 2);
        // pegawai 1: all three sizes filled, aligned to columns
        assert_eq!(a.rows[0].sizes, vec!["L", "42", "M"]);
        assert_eq!(a.rows[0].status, "J");
        assert_eq!(a.rows[0].hijab, "T");
        // pegawai 2: no sizes -> all dashes
        assert_eq!(a.rows[1].sizes, vec!["-", "-", "-"]);
        // satker B only filled Topi (index 2)
        assert_eq!(rep.satkers[1].rows[0].sizes, vec!["-", "-", "L"]);
    }

    fn rlr(
        pakaian: &str,
        group: &str,
        gender: &str,
        satker: &str,
        uk: &str,
        n: i64,
    ) -> RekapLongRow {
        RekapLongRow {
            pakaian_nama: pakaian.to_string(),
            ukuran_group: group.to_string(),
            gender: gender.to_string(),
            satker_nama: satker.to_string(),
            ukuran: uk.to_string(),
            jumlah: n,
        }
    }

    #[test]
    fn tanggal_indo_follows_the_legacy_spelling() {
        let d = |y, m, day| NaiveDate::from_ymd_opt(y, m, day);
        assert_eq!(tanggal_indo(d(2026, 11, 1)), "01-Nopember-2026");
        assert_eq!(tanggal_indo(d(2026, 12, 31)), "31-Desember-2026");
        assert_eq!(tanggal_indo(d(2026, 2, 9)), "09-Februari-2026");
        assert_eq!(tanggal_indo(None), "");
        let h = ReportHeader {
            periode_mulai: d(2026, 11, 1),
            periode_selesai: d(2026, 12, 31),
            ..Default::default()
        };
        assert_eq!(
            h.periode_line(),
            "Periode: 01-Nopember-2026 S/D 31-Desember-2026"
        );
    }

    #[test]
    fn daftar_lists_a_satker_with_nobody_in_it() {
        let rep = build_daftar_report(
            ReportHeader::default(),
            vec!["Kemeja".to_string()],
            vec!["Kejari A".to_string(), "Kejari Kosong".to_string()],
            vec![dlr("Kejari A", "1", Some("Kemeja"), Some("L"))],
        );
        let names: Vec<&str> = rep.satkers.iter().map(|s| s.satker_nama.as_str()).collect();
        assert_eq!(
            names,
            vec!["Kejari A", "Kejari Kosong"],
            "given order, empty one kept"
        );
        assert!(
            rep.satkers[1].rows.is_empty(),
            "the renderer prints Data Kosong for it"
        );
    }

    #[test]
    fn a_satker_listed_twice_is_printed_once() {
        let rep = build_daftar_report(
            ReportHeader::default(),
            vec!["Kemeja".to_string()],
            vec!["Kejari A".to_string(), "Kejari A".to_string()],
            vec![dlr("Kejari A", "1", Some("Kemeja"), Some("L"))],
        );
        assert_eq!(rep.satkers.len(), 1);
        let rekap = build_rekap_report(
            ReportHeader::default(),
            &baju_order(),
            vec![kemeja("L")],
            vec!["Kejari A".into(), "Kejari A".into()],
            vec![],
        );
        assert_eq!(rekap.blocks[0].satkers.len(), 1);
    }

    #[test]
    fn daftar_keeps_a_satker_that_only_the_rows_know() {
        let rep = build_daftar_report(
            ReportHeader::default(),
            vec!["Kemeja".to_string()],
            vec!["Kejari A".to_string()],
            vec![dlr("Kejari Z", "9", Some("Kemeja"), Some("M"))],
        );
        assert_eq!(rep.satkers.len(), 2);
        assert_eq!(rep.satkers[1].satker_nama, "Kejari Z");
    }

    fn baju_order() -> HashMap<String, Vec<String>> {
        let mut order = HashMap::new();
        order.insert(
            "BAJU".to_string(),
            ["S", "M", "L", "XL"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );
        order
    }

    fn kemeja(gender: &str) -> RekapPakaian {
        RekapPakaian {
            nama: "Kemeja".to_string(),
            ukuran_group: "BAJU".to_string(),
            gender: gender.to_string(),
        }
    }

    #[test]
    fn rekap_prints_every_size_and_every_satker_with_zeros() {
        let rows = vec![
            rlr("Kemeja", "BAJU", "L", "Kejari A", "M", 3),
            rlr("Kemeja", "BAJU", "L", "Kejari A", "L", 2),
            rlr("Kemeja", "BAJU", "L", "Kejari B", "M", 1),
            rlr("Kemeja", "BAJU", "P", "Kejari A", "S", 4),
        ];
        let rep = build_rekap_report(
            ReportHeader::default(),
            &baju_order(),
            vec![kemeja("SEMUA")],
            vec!["Kejari A".into(), "Kejari B".into(), "Kejari C".into()],
            rows,
        );

        // "SEMUA" gives both gender tables, L first.
        assert_eq!(rep.blocks.len(), 2);
        let l = &rep.blocks[0];
        assert_eq!(l.gender, "L");
        // Every size of the group, in master order, whether or not anyone picked it.
        assert_eq!(l.ukuran_labels, vec!["S", "M", "L", "XL"]);
        // Every satker, whether or not it has data: C has nothing and still has a row.
        assert_eq!(l.satkers.len(), 3);
        assert_eq!(l.satkers[0].counts, vec![0, 3, 2, 0]);
        assert_eq!(l.satkers[0].total, 5);
        assert_eq!(l.satkers[1].counts, vec![0, 1, 0, 0]);
        assert_eq!(l.satkers[2].counts, vec![0, 0, 0, 0]);
        assert_eq!(l.satkers[2].total, 0);
        assert_eq!(l.summary, vec![0, 4, 2, 0]);
        assert_eq!(l.summary_total, 6);

        let p = &rep.blocks[1];
        assert_eq!(p.gender, "P");
        assert_eq!(p.summary, vec![4, 0, 0, 0]);
        assert_eq!(p.summary_total, 4);
    }

    #[test]
    fn rekap_gender_specific_pakaian_gets_only_its_table() {
        let order = baju_order();
        let only_l = build_rekap_report(
            ReportHeader::default(),
            &order,
            vec![kemeja("L")],
            vec!["Kejari A".into()],
            vec![],
        );
        assert_eq!(only_l.blocks.len(), 1);
        assert_eq!(only_l.blocks[0].gender, "L");

        // A "SEMUA" row next to an "L" row of the same name does not print the L page twice.
        let dup = build_rekap_report(
            ReportHeader::default(),
            &order,
            vec![kemeja("L"), kemeja("SEMUA")],
            vec!["Kejari A".into()],
            vec![],
        );
        let genders: Vec<&str> = dup.blocks.iter().map(|b| b.gender.as_str()).collect();
        assert_eq!(genders, vec!["L", "P"]);
    }

    #[test]
    fn rekap_never_drops_a_size_or_satker_the_master_does_not_know() {
        let rows = vec![rlr("Kemeja", "BAJU", "L", "Kejari Z", "3XL", 2)];
        let rep = build_rekap_report(
            ReportHeader::default(),
            &baju_order(),
            vec![kemeja("L")],
            vec!["Kejari A".into()],
            rows,
        );
        let b = &rep.blocks[0];
        assert_eq!(b.ukuran_labels, vec!["S", "M", "L", "XL", "3XL"]);
        assert_eq!(b.satkers.len(), 2);
        assert_eq!(b.satkers[1].satker_nama, "Kejari Z");
        assert_eq!(b.satkers[1].counts, vec![0, 0, 0, 0, 2]);
        assert_eq!(b.summary_total, 2);
    }
}
