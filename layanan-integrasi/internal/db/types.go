// File: internal/db/types.go
package db

type Satker struct {
	ID             string `json:"id"`
	NamaSatker     string `json:"nama_satker"`
	Provinsi       string `json:"provinsi"`
	Wilayah        string `json:"wilayah"`
	KategoriSatker string `json:"kategori_satker"`
}

type Pegawai struct {
	NIP                  string `json:"nip"`
	Nama                 string `json:"nama"`
	EmailDinas           string `json:"email_dinas"`
	GolPangkat           string `json:"golpang"`
	Jabatan              string `json:"jabatan"`
	JenisJabatanTerakhir string `json:"jenis_jabatan_terakhir"`
	Agama                string `json:"agama"`
	JenisKelamin         string `json:"jk"`
	GolKd                string `json:"GOL_KD"`
	Eselon               string `json:"eselon"`
	Foto                 string `json:"foto"`
}
