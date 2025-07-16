// layanan-integrasi/cmd/main.go

package main

import (
	"context"
	"log"

	"simpelv2/layanan-integrasi/config"
	"simpelv2/layanan-integrasi/internal/db"
	"simpelv2/layanan-integrasi/internal/db/generated"
	"simpelv2/layanan-integrasi/internal/integrasi/mysimkari"
)

func main() {
	ctx := context.Background()
	config.Load() // load config.yaml → AppConfig

	dbPool := db.ConnectDB(config.AppConfig.GetDatabaseConfig())
	q := generated.New(dbPool)

	if err := mysimkari.SyncSatker(ctx, q); err != nil {
		log.Fatal("Gagal sync satker:", err)
	}
	if err := mysimkari.SyncPegawai(ctx, q); err != nil {
		log.Fatal("Gagal sync pegawai:", err)
	}

	log.Println("Sinkronisasi selesai.")
}
