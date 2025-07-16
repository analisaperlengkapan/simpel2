// layanan-integrasi/internal/integrasi/mysimkari/client.go

package mysimkari

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
)

type Pegawai struct {
	NIP                  string  `json:"nip"`
	Nama                 string  `json:"nama"`
	EmailDinas           *string `json:"email_dinas"`
	GolPangkat           *string `json:"golpang"`
	Jabatan              *string `json:"jabatan"`
	JenisJabatanTerakhir *string `json:"jenis_jabatan_terakhir"`
	Agama                *string `json:"agama"`
	JenisKelamin         *string `json:"jk"`
	GolKd                *string `json:"GOL_KD"`
	Eselon               *string `json:"eselon"`
	Foto                 *string `json:"foto"`
}

func GetPegawaiBySatker(apiURL, token string, satkerID string) ([]Pegawai, error) {
	url := fmt.Sprintf("%s/pegawai-satker/%s", apiURL, satkerID)
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return nil, err
	}
	req.Header.Add("Authorization", "Bearer "+token)

	res, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer res.Body.Close()

	body, _ := io.ReadAll(res.Body)

	var response struct {
		Status  string    `json:"status"`
		Message string    `json:"message"`
		Data    []Pegawai `json:"data"`
	}
	err = json.Unmarshal(body, &response)
	if err != nil {
		return nil, err
	}

	return response.Data, nil
}
