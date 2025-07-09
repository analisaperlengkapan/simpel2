package main

import (
	"layanan-pengguna/internal/db"
	"layanan-pengguna/internal/handler"
	"log"
	"os"

	"github.com/gofiber/fiber/v2"
	"github.com/joho/godotenv"
)

func main() {
	godotenv.Load()
	db.InitDB()

	app := fiber.New()

	app.Get("/api/pengguna/:id", handler.GetPengguna)
	app.Get("/api/pengguna", handler.ListPengguna)

	port := os.Getenv("PENGGUNA_PORT")
	if port == "" {
		port = "3000"
	}

	log.Fatal(app.Listen(":" + port))
}
