package main

import (
	"layanan-keamanan/internal/db"
	"layanan-keamanan/internal/handler"
	"log"
	"os"

	"github.com/gofiber/fiber/v2"
	"github.com/joho/godotenv"
)

func main() {
	godotenv.Load()

	app := fiber.New()

	db.InitDB()

	app.Post("/api/auth/login", handler.Login)
	app.Post("/api/auth/refresh", handler.Refresh)

	port := os.Getenv("AUTH_PORT")
	if port == "" {
		port = "3000"
	}

	log.Fatal(app.Listen(":" + port))
}
