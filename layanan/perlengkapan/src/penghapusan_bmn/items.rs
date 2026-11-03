//! The assets a penghapusan usulan lists: what the operator names, and what
//! SIMAN says they are.
//!
//! An operator names an asset the way the domain does — **kode barang + NUP**,
//! inside their own satker. Everything else about it is SIMAN's: the standard
//! name and the acquisition value. The usulan stores a copy of both (the SK has
//! to keep saying what was removed even after SIMAN changes), but the copy is
//! taken from SIMAN by the server, never from the request.
//!
//! # What was wrong
//!
//! `create` verified one asset: the "item utama" carried in the request's top
//! level fields. The `items` array — the other assets in a multi-item usulan —
//! was written to `penghapusan_bmn_item` exactly as the client sent it:
//!
//! * **no existence check**, so an SK could list an asset SIMAN has never heard
//!   of;
//! * **no satker scope**, so an operator could put another satker's asset on
//!   their own usulan (the primary was scoped; the rest never were);
//! * **a client-supplied `nilai_perolehan`**, which is the number that decides
//!   whether the Wilayah may decide the matter at all — "an operator who could
//!   type it could simply type a small one", the very comment the primary's
//!   lookup carries;
//! * and the Wilayah value ceiling looked at the primary's value alone, so a
//!   small primary carried any number of large items past it.
//!
//! The form also asked the operator to type the name, which SIMAN already knows
//! and which the picker had filled in anyway.

use std::collections::HashMap;

use super::models::{CreatePenghapusanBmnItemRequest, CreatePenghapusanBmnRequest};
use crate::bank_aset::repository::BankAsetRepository;
use crate::bank_aset::{AsetIdentity, AsetScope};
use crate::shared::error::{AppError, AppResult};
use crate::shared::siman_columns::normalize_kode_barang;

/// The most assets one usulan may list.
///
/// Not a regulatory figure — nobody has confirmed one. It bounds the work the
/// server does per request: every listed asset is one lookup over the SIMAN
/// snapshot (624 533 rows on staging), so an uncapped list is a request that
/// can keep a connection busy for as long as the body limit allows.
pub const MAX_ITEMS_PER_USULAN: usize = 200;

/// How an operator names one asset: the two parts the request legitimately
/// carries. The satker is the caller's, from their token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedAsset {
    pub kode_barang: String,
    pub nup: String,
}

impl NamedAsset {
    fn new(kode_barang: &str, nup: &str) -> Self {
        Self {
            kode_barang: kode_barang.trim().to_string(),
            nup: nup.trim().to_string(),
        }
    }

    /// Two spellings of one barang code are one asset (`3.06.02.01.003` and
    /// `3060201003`); NUP is compared as written.
    fn key(&self) -> (String, String) {
        (normalize_kode_barang(&self.kode_barang), self.nup.clone())
    }
}

/// An asset as SIMAN holds it. The value is not optional: with no acquisition
/// value the usulan cannot be placed within an authority, so the request is
/// refused rather than stored with a blank.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifiedAsset {
    pub kode_barang: String,
    pub nup: String,
    pub nama_barang: String,
    pub nilai_perolehan: f64,
}

/// The assets the request names, in order, after the checks that need no
/// database: the list is within bounds, every entry is complete, none is listed
/// twice, and — when `items` is present — its first entry is the "item utama" the
/// top-level fields describe.
///
/// That last rule closes a gap in the other direction: the parent row carries
/// the first asset's identity, so a body whose top level named one asset and
/// whose list started with another would store a usulan that describes two
/// different things depending on which table you read.
pub fn named_assets(request: &CreatePenghapusanBmnRequest) -> AppResult<Vec<NamedAsset>> {
    let primary = NamedAsset::new(&request.kode_barang, &request.nup);

    let named: Vec<NamedAsset> = if request.items.is_empty() {
        vec![primary]
    } else {
        let list: Vec<NamedAsset> = request
            .items
            .iter()
            .map(|i| NamedAsset::new(&i.kode_barang, &i.nup))
            .collect();
        if list.first().map(NamedAsset::key) != Some(primary.key()) {
            return Err(AppError::BadRequest(
                "BMN utama harus sama dengan item pertama pada daftar BMN".to_string(),
            ));
        }
        list
    };

    if named.len() > MAX_ITEMS_PER_USULAN {
        return Err(AppError::BadRequest(format!(
            "Satu usulan paling banyak memuat {MAX_ITEMS_PER_USULAN} BMN"
        )));
    }

    let mut seen: HashMap<(String, String), usize> = HashMap::new();
    for (idx, asset) in named.iter().enumerate() {
        let n = idx + 1;
        if asset.kode_barang.is_empty() || asset.nup.is_empty() {
            return Err(AppError::BadRequest(format!(
                "BMN ke-{n}: kode barang dan NUP wajib diisi"
            )));
        }
        if asset.kode_barang.len() > 50 || asset.nup.len() > 50 {
            return Err(AppError::BadRequest(format!(
                "BMN ke-{n}: kode barang dan NUP paling banyak 50 karakter"
            )));
        }
        if let Some(first) = seen.insert(asset.key(), n) {
            return Err(AppError::BadRequest(format!(
                "BMN ke-{n} sama dengan BMN ke-{first} (kode barang {}, NUP {})",
                asset.kode_barang, asset.nup
            )));
        }
    }
    Ok(named)
}

/// Resolves every named asset against SIMAN, inside the caller's own satker.
///
/// All or nothing: one asset that does not resolve refuses the whole usulan, and
/// the message says which one, so the operator is not left to bisect a list.
pub async fn verify_assets(
    bank: &BankAsetRepository,
    named: &[NamedAsset],
    scope: &AsetScope,
) -> AppResult<Vec<VerifiedAsset>> {
    let mut verified = Vec::with_capacity(named.len());
    for (idx, asset) in named.iter().enumerate() {
        let n = idx + 1;
        let found = bank
            .find_lookup_by_identity(
                AsetIdentity {
                    nup: &asset.nup,
                    kode_barang: Some(&asset.kode_barang),
                    satker_code: None,
                },
                scope,
            )
            .await
            .map_err(|e| {
                // A SIMAN failure refuses the request. Falling back to what the
                // client sent — as this once did for the value — is trusting the
                // very input this module exists not to trust.
                tracing::warn!("penghapusan: SIMAN lookup failed for item {n}: {e}");
                AppError::Internal(
                    "Data SIMAN belum dapat diverifikasi; usulan tidak dapat dibuat sekarang. \
                     Coba lagi beberapa saat."
                        .to_string(),
                )
            })?
            .ok_or_else(|| {
                AppError::BadRequest(format!(
                    "BMN ke-{n} (kode barang {}, NUP {}) tidak ditemukan di SIMAN untuk satker \
                     Anda. Pastikan kode barang dan NUP cocok dengan data SIMAN.",
                    asset.kode_barang, asset.nup
                ))
            })?;

        let nama_barang = found.nama_barang.ok_or_else(|| {
            AppError::BadRequest(format!(
                "BMN ke-{n} (kode barang {}, NUP {}) tidak memiliki nama barang di SIMAN",
                asset.kode_barang, asset.nup
            ))
        })?;
        let nilai_perolehan = found.nilai_perolehan.ok_or_else(|| {
            AppError::BadRequest(format!(
                "BMN ke-{n} (kode barang {}, NUP {}) belum memiliki nilai perolehan di SIMAN, \
                 sehingga usulan tidak dapat ditempatkan pada kewenangan penetap",
                asset.kode_barang, asset.nup
            ))
        })?;

        verified.push(VerifiedAsset {
            kode_barang: asset.kode_barang.clone(),
            nup: asset.nup.clone(),
            nama_barang,
            nilai_perolehan,
        });
    }
    Ok(verified)
}

/// What the request becomes once SIMAN has spoken: the top-level fields and the
/// item list both rebuilt from the verified assets. Whatever name or value the
/// client sent is gone; only `kondisi` — free text about the unit's state,
/// which SIMAN does not hold in a form the SK uses — is carried over, by
/// position.
pub fn apply_verified(
    mut request: CreatePenghapusanBmnRequest,
    verified: &[VerifiedAsset],
) -> CreatePenghapusanBmnRequest {
    let kondisi: Vec<Option<String>> = request.items.iter().map(|i| i.kondisi.clone()).collect();

    if let Some(first) = verified.first() {
        request.kode_barang = first.kode_barang.clone();
        request.nama_barang = first.nama_barang.clone();
        request.nup = first.nup.clone();
        request.nilai_perolehan = Some(first.nilai_perolehan);
    }
    request.items = verified
        .iter()
        .enumerate()
        .map(|(idx, v)| CreatePenghapusanBmnItemRequest {
            kode_barang: v.kode_barang.clone(),
            nama_barang: v.nama_barang.clone(),
            nup: v.nup.clone(),
            nilai_perolehan: Some(v.nilai_perolehan),
            kondisi: kondisi.get(idx).cloned().flatten(),
        })
        .collect();
    request
}

/// The value the authority ceiling is judged on: everything the usulan removes.
///
/// For one asset it is that asset's value, exactly what the ceiling saw before.
/// For several it is their sum — the Wilayah deciding on a package whose parts
/// are each under the line and whose total is far over it is the evasion the
/// per-primary check allowed.
pub fn total_nilai(verified: &[VerifiedAsset]) -> f64 {
    verified.iter().map(|v| v.nilai_perolehan).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(kode: &str, nup: &str, items: &[(&str, &str)]) -> CreatePenghapusanBmnRequest {
        serde_json::from_value(serde_json::json!({
            "kode_barang": kode,
            "nup": nup,
            "tanggal_penghapusan": "2026-01-01",
            "alasan": "Rusak berat dan tidak ekonomis untuk diperbaiki",
            "metode_penghapusan": "Pemusnahan",
            "lampiran_persyaratan": "https://storage.example.com/x.pdf",
            "items": items
                .iter()
                .map(|(k, n)| serde_json::json!({"kode_barang": k, "nup": n}))
                .collect::<Vec<_>>(),
        }))
        .unwrap()
    }

    fn msg(e: AppError) -> String {
        match e {
            AppError::BadRequest(m) => m,
            other => panic!("expected BadRequest, got {other:?}"),
        }
    }

    #[test]
    fn a_name_is_not_required_of_the_client() {
        // `nama_barang` is absent from the body above; before, that was a 400.
        let r = request("3060201003", "15", &[]);
        assert_eq!(r.nama_barang, "");
        assert_eq!(named_assets(&r).unwrap().len(), 1);
    }

    #[test]
    fn a_single_asset_is_the_primary() {
        let named = named_assets(&request("3060201003", "15", &[])).unwrap();
        assert_eq!(named, vec![NamedAsset::new("3060201003", "15")]);
    }

    #[test]
    fn the_list_must_start_with_the_primary() {
        let r = request(
            "3060201003",
            "15",
            &[("3060201004", "16"), ("3060201003", "15")],
        );
        assert!(msg(named_assets(&r).unwrap_err()).contains("item pertama"));
    }

    #[test]
    fn the_primary_may_be_spelled_with_dots() {
        let r = request(
            "3.06.02.01.003",
            "15",
            &[("3060201003", "15"), ("3060201004", "16")],
        );
        assert_eq!(named_assets(&r).unwrap().len(), 2);
    }

    #[test]
    fn the_same_asset_twice_is_refused_whatever_its_spelling() {
        let r = request(
            "3060201003",
            "15",
            &[("3060201003", "15"), ("3.06.02.01.003", "15")],
        );
        let m = msg(named_assets(&r).unwrap_err());
        assert!(m.contains("ke-2") && m.contains("ke-1"), "{m}");
    }

    #[test]
    fn the_same_code_with_another_nup_is_another_asset() {
        let r = request(
            "3060201003",
            "15",
            &[("3060201003", "15"), ("3060201003", "99")],
        );
        assert_eq!(named_assets(&r).unwrap().len(), 2);
    }

    #[test]
    fn an_incomplete_entry_names_its_position() {
        let r = request("3060201003", "15", &[("3060201003", "15"), ("", "16")]);
        assert!(msg(named_assets(&r).unwrap_err()).contains("ke-2"));
        let r = request("3060201003", "  ", &[]);
        assert!(msg(named_assets(&r).unwrap_err()).contains("ke-1"));
    }

    #[test]
    fn the_list_is_bounded() {
        let many: Vec<(String, String)> = (0..=MAX_ITEMS_PER_USULAN)
            .map(|n| ("3060201003".to_string(), n.to_string()))
            .collect();
        let borrowed: Vec<(&str, &str)> =
            many.iter().map(|(k, n)| (k.as_str(), n.as_str())).collect();
        let r = request("3060201003", "0", &borrowed);
        assert!(msg(named_assets(&r).unwrap_err()).contains("paling banyak"));
        let ok = &borrowed[..MAX_ITEMS_PER_USULAN];
        assert!(named_assets(&request("3060201003", "0", ok)).is_ok());
    }

    #[test]
    fn what_the_client_said_about_names_and_values_does_not_survive() {
        let mut r = request(
            "3060201003",
            "15",
            &[("3060201003", "15"), ("3060201004", "16")],
        );
        r.nama_barang = "Kapal Selam".to_string();
        r.nilai_perolehan = Some(1.0);
        r.items[0].nama_barang = "Kapal Selam".to_string();
        r.items[0].nilai_perolehan = Some(1.0);
        r.items[1].nama_barang = "Pesawat Tempur".to_string();
        r.items[1].nilai_perolehan = Some(2.0);
        r.items[1].kondisi = Some("Rusak berat".to_string());

        let verified = vec![
            VerifiedAsset {
                kode_barang: "3060201003".into(),
                nup: "15".into(),
                nama_barang: "Pesawat Telephone".into(),
                nilai_perolehan: 12_500_000.0,
            },
            VerifiedAsset {
                kode_barang: "3060201004".into(),
                nup: "16".into(),
                nama_barang: "Telephone Mobile".into(),
                nilai_perolehan: 4_300_000.0,
            },
        ];
        let out = apply_verified(r, &verified);

        assert_eq!(out.nama_barang, "Pesawat Telephone");
        assert_eq!(out.nilai_perolehan, Some(12_500_000.0));
        assert_eq!(out.items[0].nama_barang, "Pesawat Telephone");
        assert_eq!(out.items[1].nama_barang, "Telephone Mobile");
        assert_eq!(out.items[1].nilai_perolehan, Some(4_300_000.0));
        // Free text about the unit's state is the operator's own.
        assert_eq!(out.items[1].kondisi.as_deref(), Some("Rusak berat"));
        assert_eq!(out.items[0].kondisi, None);
    }

    #[test]
    fn a_single_asset_request_gains_a_one_entry_list() {
        let verified = vec![VerifiedAsset {
            kode_barang: "3060201003".into(),
            nup: "15".into(),
            nama_barang: "Pesawat Telephone".into(),
            nilai_perolehan: 12_500_000.0,
        }];
        let out = apply_verified(request("3060201003", "15", &[]), &verified);
        assert_eq!(out.items.len(), 1);
        assert_eq!(out.items[0].nup, "15");
    }

    #[test]
    fn the_ceiling_is_judged_on_the_total_not_the_first() {
        let a = |nilai| VerifiedAsset {
            kode_barang: "k".into(),
            nup: "1".into(),
            nama_barang: "n".into(),
            nilai_perolehan: nilai,
        };
        assert_eq!(total_nilai(&[a(10.0)]), 10.0);
        assert_eq!(total_nilai(&[a(10.0), a(990.0), a(1.0)]), 1001.0);
    }
}
