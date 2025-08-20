// layanan-integrasi/internal/integrasi/service.go

package integrasi

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"net/http"
	"time"

	"layanan-integrasi/config"
)

type Service struct {
	cfg *config.Config
	db  *sql.DB
}

func NewService(cfg *config.Config, db *sql.DB) *Service {
	return &Service{cfg: cfg, db: db}
}

type Satker struct {
	ID         string `json:"id"`
	NamaSatker string `json:"nama_satker"`
	Provinsi   string `json:"provinsi"`
	Wilayah    string `json:"wilayah"`
	Kategori   string `json:"kategori_satker"`
	// Tambahkan jika ada kolom tambahan lain
}

type Pegawai struct {
	NIP   string `json:"nip"`
	Nama  string `json:"nama"`
	Email string `json:"email_dinas"`
	Jab   string `json:"jabatan"`
	Foto  string `json:"foto"`
}

func (s *Service) TarikSatker() error {
	url := fmt.Sprintf("%s/get-satker", s.cfg.MySimkari.BaseURL)
	req, _ := http.NewRequest("GET", url, nil)
	req.Header.Set("Authorization", "Bearer "+s.cfg.MySimkari.Token)

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	var result struct {
		Data []Satker `json:"data"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return err
	}

	tx, err := s.db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	if _, err := tx.Exec("DELETE FROM mysimkari_satker"); err != nil {
		return err
	}

	stmt, err := tx.Prepare(`
        INSERT INTO mysimkari_satker (id, nama_satker, provinsi, wilayah, kategori_satker)
        VALUES ($1, $2, $3, $4, $5)
    `)
	if err != nil {
		return err
	}
	defer stmt.Close()

	for _, satker := range result.Data {
		_, err := stmt.Exec(satker.ID, satker.NamaSatker, satker.Provinsi, satker.Wilayah, satker.Kategori)
		if err != nil {
			return err
		}
	}

	return tx.Commit()
}

func (s *Service) TarikPegawai() error {
	rows, err := s.db.Query("SELECT id FROM mysimkari_satker")
	if err != nil {
		return err
	}
	defer rows.Close()

	for rows.Next() {
		var id string
		if err := rows.Scan(&id); err != nil {
			return err
		}

		err := s.syncPegawaiBySatker(id)
		if err != nil {
			fmt.Printf("Gagal tarik pegawai satker %s: %v\n", id, err)
		}

		time.Sleep(2 * time.Second) // throttling antar permintaan
	}

	return nil
}

func (s *Service) syncPegawaiBySatker(id string) error {
	url := fmt.Sprintf("%s/pegawai-satker/%s", s.cfg.MySimkari.BaseURL, id)
	req, _ := http.NewRequest("GET", url, nil)
	req.Header.Set("Authorization", "Bearer "+s.cfg.MySimkari.Token)

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	var result struct {
		Data []Pegawai `json:"data"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return err
	}

	tx, err := s.db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	_, err = tx.Exec("DELETE FROM mv_curr_pegawai_all WHERE mysimkari_satker_id = $1", id)
	if err != nil {
		return err
	}

	stmt, err := tx.Prepare(`
        INSERT INTO mv_curr_pegawai_all (peg_nip_baru, nama, pns_mail, jabatan, foto, mysimkari_satker_id)
        VALUES ($1, $2, $3, $4, $5, $6)
    `)
	if err != nil {
		return err
	}
	defer stmt.Close()

	for _, p := range result.Data {
		_, err := stmt.Exec(p.NIP, p.Nama, p.Email, p.Jab, p.Foto, id)
		if err != nil {
			return err
		}
	}

	return tx.Commit()
}
