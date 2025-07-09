package db

import (
	"context"
	"fmt"
	"os"

	"github.com/jackc/pgx/v5"
)

var Conn *pgx.Conn

func InitDB() {
	url := os.Getenv("DATABASE_URL")
	if url == "" {
		url = "postgres://postgres:rahasia123@db-simpelv2:5432/simpelv2"
	}

	var err error
	Conn, err = pgx.Connect(context.Background(), url)
	if err != nil {
		panic(fmt.Sprintf("Unable to connect to DB: %v", err))
	}
}
