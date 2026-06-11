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
    pub periode_mulai: Option<NaiveDate>,
    pub periode_selesai: Option<NaiveDate>,
    /// Applied filters as (label, value) pairs (legacy `$filter`).
    pub filters: Vec<(String, String)>,
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
    rows: Vec<DaftarLongRow>,
) -> DaftarReport {
    let col_index: HashMap<&str, usize> = columns
        .iter()
        .enumerate()
        .map(|(i, c)| (c.as_str(), i))
        .collect();
    let n_cols = columns.len();

    let mut satker_order: Vec<String> = Vec::new();
    // satker_nama -> (nip order, nip -> accum)
    let mut satkers: HashMap<String, (Vec<String>, HashMap<String, DaftarPegawaiAccum>)> =
        HashMap::new();

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

/// Pivot raw long rows into per-(pakaian × gender) blocks with satker rows and
/// a summary row.
///
/// `ukuran_order` maps an `ukuran_group` to its master-ordered label list
/// (from the `ukuran` master, by `urutan`). Within a block, columns are those
/// master labels that actually occur, in master order; any size present but
/// absent from the master list is appended (sorted) so no data is dropped.
pub fn build_rekap_report(
    header: ReportHeader,
    ukuran_order: &HashMap<String, Vec<String>>,
    rows: Vec<RekapLongRow>,
) -> RekapReport {
    // Block identity = (pakaian_nama, ukuran_group, gender), first-seen order.
    let mut block_order: Vec<(String, String, String)> = Vec::new();
    let mut block_rows: HashMap<(String, String, String), Vec<RekapLongRow>> = HashMap::new();
    for row in rows {
        let key = (
            row.pakaian_nama.clone(),
            row.ukuran_group.clone(),
            row.gender.clone(),
        );
        block_rows
            .entry(key.clone())
            .or_insert_with(|| {
                block_order.push(key.clone());
                Vec::new()
            })
            .push(row);
    }

    let blocks = block_order
        .into_iter()
        .map(|key| {
            let (pakaian_nama, ukuran_group, gender) = key.clone();
            let rows = block_rows.remove(&key).expect("block present");

            // Labels: master order filtered to those present, then any extras.
            let present: HashSet<&str> = rows.iter().map(|r| r.ukuran.as_str()).collect();
            let mut labels: Vec<String> = ukuran_order
                .get(&ukuran_group)
                .map(|all| {
                    all.iter()
                        .filter(|u| present.contains(u.as_str()))
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();
            let mut extras: Vec<String> = present
                .iter()
                .filter(|u| !labels.iter().any(|l| l == *u))
                .map(|u| u.to_string())
                .collect();
            extras.sort();
            labels.extend(extras);

            let label_index: HashMap<&str, usize> = labels
                .iter()
                .enumerate()
                .map(|(i, l)| (l.as_str(), i))
                .collect();
            let n = labels.len();

            let mut satker_order: Vec<String> = Vec::new();
            let mut satker_counts: HashMap<String, Vec<i64>> = HashMap::new();
            for r in &rows {
                let counts = satker_counts
                    .entry(r.satker_nama.clone())
                    .or_insert_with(|| {
                        satker_order.push(r.satker_nama.clone());
                        vec![0; n]
                    });
                if let Some(&idx) = label_index.get(r.ukuran.as_str()) {
                    counts[idx] += r.jumlah;
                }
            }

            let mut summary = vec![0i64; n];
            let satkers: Vec<RekapSatkerRow> = satker_order
                .into_iter()
                .map(|satker_nama| {
                    let counts = satker_counts.remove(&satker_nama).expect("satker present");
                    let total: i64 = counts.iter().sum();
                    for (i, c) in counts.iter().enumerate() {
                        summary[i] += c;
                    }
                    RekapSatkerRow {
                        satker_nama,
                        counts,
                        total,
                    }
                })
                .collect();
            let summary_total: i64 = summary.iter().sum();

            RekapBlock {
                pakaian_nama,
                ukuran_group,
                gender,
                ukuran_labels: labels,
                satkers,
                summary,
                summary_total,
            }
        })
        .collect();

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
        let rep = build_daftar_report(ReportHeader::default(), columns, rows);

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
    fn rekap_blocks_per_pakaian_gender_with_satker_rows_and_summary() {
        let mut order = HashMap::new();
        order.insert(
            "BAJU".to_string(),
            vec!["S".to_string(), "M".to_string(), "L".to_string()],
        );
        let rows = vec![
            rlr("Kemeja", "BAJU", "L", "Kejari A", "M", 3),
            rlr("Kemeja", "BAJU", "L", "Kejari A", "L", 2),
            rlr("Kemeja", "BAJU", "L", "Kejari B", "M", 1),
            rlr("Kemeja", "BAJU", "P", "Kejari A", "S", 4),
        ];
        let rep = build_rekap_report(ReportHeader::default(), &order, rows);

        // Two blocks: (Kemeja,L) and (Kemeja,P)
        assert_eq!(rep.blocks.len(), 2);
        let l = &rep.blocks[0];
        assert_eq!(l.gender, "L");
        // master order S,M,L filtered to present {M,L} -> [M, L]
        assert_eq!(l.ukuran_labels, vec!["M", "L"]);
        assert_eq!(l.satkers.len(), 2);
        // Kejari A: M=3, L=2, total 5
        assert_eq!(l.satkers[0].counts, vec![3, 2]);
        assert_eq!(l.satkers[0].total, 5);
        // Kejari B: M=1, L=0
        assert_eq!(l.satkers[1].counts, vec![1, 0]);
        // column summary M=4, L=2; grand 6
        assert_eq!(l.summary, vec![4, 2]);
        assert_eq!(l.summary_total, 6);

        let p = &rep.blocks[1];
        assert_eq!(p.gender, "P");
        assert_eq!(p.ukuran_labels, vec!["S"]);
        assert_eq!(p.summary_total, 4);
    }
}
