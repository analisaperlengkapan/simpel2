package main

import (
	"layanan-audit/internal/db"
	"layanan-audit/internal/handler"
	"os"

	"github.com/gofiber/fiber/v2"
	"github.com/joho/godotenv"
)

func main() {
	godotenv.Load()
	db.InitDB()

	app := fiber.New()

	app.Post("/api/audit", handler.LogAudit)

	port := os.Getenv("AUDIT_PORT")
	if port == "" {
		port = "3000"
	}

	app.Listen(":" + port)
}
