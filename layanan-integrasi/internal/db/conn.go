// layanan-integrasi/internal/db/conn.go

package db

import (
	"context"
	"fmt"
	"log"

	"simpelv2/layanan-integrasi/config"

	"github.com/jackc/pgx/v5/pgxpool"
)

var Pool *pgxpool.Pool

func ConnectDB(cfg config.DatabaseConfig) *pgxpool.Pool {
	connStr := fmt.Sprintf(
		"postgres://%s:%s@%s:%d/%s?sslmode=require",
		cfg.User, cfg.Password, cfg.Host, cfg.Port, cfg.Name,
	)

	pool, err := pgxpool.New(context.Background(), connStr)
	if err != nil {
		log.Fatalf("Gagal membuat koneksi pool: %v", err)
	}

	if err := pool.Ping(context.Background()); err != nil {
		log.Fatalf("Gagal ping database: %v", err)
	}

	Pool = pool
	return pool
}
