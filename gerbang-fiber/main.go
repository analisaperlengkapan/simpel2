package main

import (
	"log"
	"os"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"github.com/gofiber/fiber/v2/middleware/limiter"
	"github.com/joho/godotenv"

	"simpelv2/gerbang-fiber/internal/middleware"
	"simpelv2/gerbang-fiber/internal/proxy"
)

func main() {
	// Load env
	if err := godotenv.Load(); err != nil {
		log.Println("INFO: .env file not found, proceeding with environment variables")
	}

	// Init Fiber
	app := fiber.New(fiber.Config{
		ServerHeader: "Simpelv2-Gateway",
		AppName:      "Simpelv2 Fiber Gateway",
		IdleTimeout:  10 * time.Second,
	})

	// Middleware CORS
	app.Use(cors.New(cors.Config{
		AllowOrigins: "*",
		AllowHeaders: "Origin, Content-Type, Accept, Authorization",
	}))

	// Middleware Rate Limiting
	app.Use(limiter.New(limiter.Config{
		Max:        100,
		Expiration: 1 * time.Minute,
	}))

	// Middleware Logging to Audit
	app.Use(middleware.AuditLogger())

	// Middleware Auth (except open routes)
	app.Use(middleware.AuthExcept([]string{
		"/api/auth/login",
		"/api/auth/refresh",
		"/api/auth/mfa/request",
		"/api/auth/mfa/verify",
	}))

	// Routing: Proxy all requests to services
	proxy.SetupRoutes(app)

	// Start server
	port := os.Getenv("GATEWAY_PORT")
	if port == "" {
		port = "8080"
	}

	log.Printf("✅ Gateway Fiber running at :%s", port)
	if err := app.Listen(":" + port); err != nil {
		log.Fatalf("❌ Failed to start server: %v", err)
	}
}
