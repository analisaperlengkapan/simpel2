// layanan-integrasi/internal/integrasi/api_client.go

package mysimkari

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"

	"simpelv2/layanan-integrasi/config"
	"simpelv2/layanan-integrasi/internal/db"
)

// Tidak perlu baseURL dan token sebagai variabel global karena sudah ada di AppConfig

func fetchJSON(endpoint string, result any) error {
	baseURL := config.AppConfig.MySimkari.URL
	token := config.AppConfig.MySimkari.Token

	// Validasi token
	if token == "" {
		return errors.New("token MySimkari belum dikonfigurasi")
	}

	url := fmt.Sprintf("%s/%s", baseURL, endpoint)

	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		return err
	}
	req.Header.Set("Authorization", "Bearer "+token)
	req.Header.Set("Content-Type", "application/json")

	res, err := http.DefaultClient.Do(req)
	if err != nil {
		return err
	}
	defer res.Body.Close()

	if res.StatusCode != http.StatusOK {
		return fmt.Errorf("status code %d", res.StatusCode)
	}

	body, err := io.ReadAll(res.Body)
	if err != nil {
		return err
	}

	if err := json.Unmarshal(body, result); err != nil {
		return errors.New("gagal parsing JSON: " + err.Error())
	}

	return nil
}

func FetchSatker() ([]db.Satker, error) {
	var response struct {
		Status  string      `json:"status"`
		Message string      `json:"message"`
		Data    []db.Satker `json:"data"`
	}
	err := fetchJSON("get-satker", &response)
	return response.Data, err
}

func FetchPegawaiBySatker(satkerID string) ([]db.Pegawai, error) {
	var response struct {
		Status  string       `json:"status"`
		Message string       `json:"message"`
		Data    []db.Pegawai `json:"data"`
	}
	err := fetchJSON(fmt.Sprintf("pegawai-satker/%s", satkerID), &response)
	return response.Data, err
}
