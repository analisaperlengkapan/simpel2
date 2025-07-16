// layanan-keamanan/seed/seed.go

package main

import (
	"context"
	"fmt"
	"log"
	"os"

	"github.com/jackc/pgx/v5"
	"golang.org/x/crypto/bcrypt"
)

func main() {
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		log.Fatal("❌ DATABASE_URL belum diset di .env")
	}

	conn, err := pgx.Connect(context.Background(), dbURL)
	if err != nil {
		log.Fatal("DB connect error: ", err)
	}
	defer conn.Close(context.Background())

	password := "admin123"
	hashed, _ := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)

	_, err = conn.Exec(context.Background(), `
		INSERT INTO auth.users (username, password)
		VALUES ($1, $2)
		ON CONFLICT (username) DO NOTHING
	`, "admin", string(hashed))

	if err != nil {
		log.Fatal("Insert failed: ", err)
	}

	fmt.Println("✅ Seed user admin berhasil (admin/admin123)")
}
