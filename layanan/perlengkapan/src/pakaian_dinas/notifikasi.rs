//! What to tell whom when a pakaian dinas satker changes hands.
//!
//! Each move of the workflow hands the work to somebody: the operator's
//! submission lands on the wilayah validator, the wilayah's forward on pusat, and
//! a return lands back on whoever sent it. Until now nobody was told, so work
//! sat in a list until the next role happened to open it. The mapping is a pure
//! function of the move so it can be tested without a database.

use crate::contracts::NotificationPriority;

/// One notification: which role receives it and what it says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PesanTransisi {
    /// The role whose turn it now is (`operator_satker`, `validator_wilayah`,
    /// `validator_pusat`).
    pub peran_tujuan: &'static str,
    pub event: &'static str,
    pub judul: String,
    pub isi: String,
    pub prioritas: NotificationPriority,
}

/// The notification for a satker moving to `ke`, or `None` for a move nobody is
/// waiting on. `catatan` is the validator's note on a return.
pub fn pesan_untuk_transisi(
    ke: i32,
    satker_nama: &str,
    catatan: Option<&str>,
) -> Option<PesanTransisi> {
    let catatan = catatan.map(str::trim).filter(|c| !c.is_empty());
    let dengan_catatan = |dasar: String| match catatan {
        Some(c) => format!("{dasar} Catatan: {c}"),
        None => dasar,
    };
    let pesan = match ke {
        // Submitted: the wilayah validator reviews it.
        1001 => PesanTransisi {
            peran_tujuan: "validator_wilayah",
            event: "pakaian_dinas.diajukan",
            judul: "Pakaian dinas menunggu pemeriksaan Anda".to_string(),
            isi: format!(
                "{satker_nama} mengajukan ukuran pakaian dinas dan menunggu pemeriksaan Anda."
            ),
            prioritas: NotificationPriority::Medium,
        },
        // Forwarded: pusat approves it.
        1004 => PesanTransisi {
            peran_tujuan: "validator_pusat",
            event: "pakaian_dinas.diteruskan",
            judul: "Pakaian dinas menunggu persetujuan Pusat".to_string(),
            isi: format!("{satker_nama} sudah diteruskan Wilayah dan menunggu persetujuan Anda."),
            prioritas: NotificationPriority::Medium,
        },
        // Returned for revision: the operator corrects it.
        1003 => PesanTransisi {
            peran_tujuan: "operator_satker",
            event: "pakaian_dinas.dikembalikan",
            judul: "Pakaian dinas dikembalikan untuk diperbaiki".to_string(),
            isi: dengan_catatan(format!(
                "{satker_nama} dikembalikan Wilayah untuk diperbaiki."
            )),
            prioritas: NotificationPriority::High,
        },
        // Sent back by pusat: the wilayah validator answers it.
        1007 => PesanTransisi {
            peran_tujuan: "validator_wilayah",
            event: "pakaian_dinas.dikembalikan_pusat",
            judul: "Pakaian dinas dikembalikan oleh Pusat".to_string(),
            isi: dengan_catatan(format!("Pusat mengembalikan {satker_nama} kepada Wilayah.")),
            prioritas: NotificationPriority::High,
        },
        // Done: the operator who filled it learns it is final.
        1008 => PesanTransisi {
            peran_tujuan: "operator_satker",
            event: "pakaian_dinas.selesai",
            judul: "Pakaian dinas disetujui".to_string(),
            isi: format!("{satker_nama} disetujui Pusat dan dinyatakan selesai."),
            prioritas: NotificationPriority::Medium,
        },
        _ => return None,
    };
    Some(pesan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_handover_goes_to_the_role_whose_turn_it_is() {
        let tujuan = |ke| pesan_untuk_transisi(ke, "KEJARI A", None).map(|p| p.peran_tujuan);
        assert_eq!(
            tujuan(1001),
            Some("validator_wilayah"),
            "submitted -> wilayah"
        );
        assert_eq!(tujuan(1004), Some("validator_pusat"), "forwarded -> pusat");
        assert_eq!(
            tujuan(1003),
            Some("operator_satker"),
            "returned -> operator"
        );
        assert_eq!(
            tujuan(1007),
            Some("validator_wilayah"),
            "returned by pusat -> wilayah"
        );
        assert_eq!(tujuan(1008), Some("operator_satker"), "done -> operator");
    }

    #[test]
    fn a_state_nobody_waits_on_says_nothing() {
        for ke in [1000, 1005, 1006, 1009, 2004] {
            assert_eq!(pesan_untuk_transisi(ke, "KEJARI A", None), None, "{ke}");
        }
    }

    #[test]
    fn a_return_quotes_the_validators_note_and_names_the_satker() {
        let p =
            pesan_untuk_transisi(1003, "KEJARI A", Some("  Ukuran celana tidak sesuai  ")).unwrap();
        assert!(p.isi.contains("KEJARI A"), "{}", p.isi);
        assert!(
            p.isi.contains("Catatan: Ukuran celana tidak sesuai"),
            "{}",
            p.isi
        );
        assert_eq!(p.prioritas, NotificationPriority::High);
        // No note, no dangling "Catatan:".
        let tanpa = pesan_untuk_transisi(1003, "KEJARI A", Some("   ")).unwrap();
        assert!(!tanpa.isi.contains("Catatan"), "{}", tanpa.isi);
    }

    #[test]
    fn events_are_distinct_so_a_template_can_tell_them_apart() {
        let events: std::collections::HashSet<_> = [1001, 1004, 1003, 1007, 1008]
            .into_iter()
            .filter_map(|ke| pesan_untuk_transisi(ke, "X", None))
            .map(|p| p.event)
            .collect();
        assert_eq!(events.len(), 5);
    }
}
