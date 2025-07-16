// File: internal/integrasi/mysimkari/sync.go
package mysimkari

import (
	"context"
	"log"

	"simpelv2/layanan-integrasi/internal/db/generated"

	"github.com/jackc/pgx/v5/pgtype"
)

func SyncSatker(ctx context.Context, q *generated.Queries) error {
	log.Println("[MySimkari] Sinkronisasi Satker dimulai...")

	satkers, err := FetchSatker()
	if err != nil {
		return err
	}

	if err := q.DeleteAllSatker(ctx); err != nil {
		return err
	}

	for _, s := range satkers {
		err := q.InsertSatker(ctx, generated.InsertSatkerParams{
			ID:             s.ID,
			NamaSatker:     s.NamaSatker,
			Provinsi:       pgtype.Text{String: s.Provinsi, Valid: s.Provinsi != ""},
			Wilayah:        pgtype.Text{String: s.Wilayah, Valid: s.Wilayah != ""},
			KategoriSatker: pgtype.Text{String: s.KategoriSatker, Valid: s.KategoriSatker != ""},
		})
		if err != nil {
			log.Printf("Gagal insert satker %s: %v", s.ID, err)
		}
	}

	log.Printf("[MySimkari] Sinkronisasi Satker selesai. Total: %d", len(satkers))
	return nil
}

func SyncPegawai(ctx context.Context, q *generated.Queries) error {
	log.Println("[MySimkari] Sinkronisasi Pegawai dimulai...")

	satkers, err := q.GetAllSatker(ctx)
	if err != nil {
		return err
	}

	for i, satker := range satkers {
		log.Printf("[%d/%d] Satker: %s", i+1, len(satkers), satker.ID)

		err := q.TruncatePegawaiBySatker(ctx, pgtype.Text{String: satker.ID, Valid: true})
		if err != nil {
			log.Printf("Gagal truncate pegawai satker %s: %v", satker.ID, err)
			continue
		}

		pegawais, err := FetchPegawaiBySatker(satker.ID)
		if err != nil {
			log.Printf("Gagal tarik data pegawai: %v", err)
			continue
		}

		params := make([]generated.InsertPegawaiParams, 0, len(pegawais))
		for _, p := range pegawais {
			params = append(params, generated.InsertPegawaiParams{
				Nip:                  p.NIP,
				Nama:                 p.Nama,
				EmailDinas:           pgtype.Text{String: p.EmailDinas, Valid: p.EmailDinas != ""},
				Golpang:              pgtype.Text{String: p.GolPangkat, Valid: p.GolPangkat != ""},
				Jabatan:              pgtype.Text{String: p.Jabatan, Valid: p.Jabatan != ""},
				JenisJabatanTerakhir: pgtype.Text{String: p.JenisJabatanTerakhir, Valid: p.JenisJabatanTerakhir != ""},
				Agama:                pgtype.Text{String: p.Agama, Valid: p.Agama != ""},
				JenisKelamin:         pgtype.Text{String: p.JenisKelamin, Valid: p.JenisKelamin != ""},
				GolKd:                pgtype.Text{String: p.GolKd, Valid: p.GolKd != ""},
				Eselon:               pgtype.Text{String: p.Eselon, Valid: p.Eselon != ""},
				Foto:                 pgtype.Text{String: p.Foto, Valid: p.Foto != ""},
				SatkerID:             pgtype.Text{String: satker.ID, Valid: true},
			})
		}

		_, err = q.InsertPegawai(ctx, params)
		if err != nil {
			log.Printf("Gagal insert pegawai satker %s: %v", satker.ID, err)
			continue
		}
	}

	log.Println("[MySimkari] Sinkronisasi Pegawai selesai.")
	return nil
}
