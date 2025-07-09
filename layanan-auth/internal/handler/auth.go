package handler

import (
	"context"
	"layanan-auth/internal/db"
	"os"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/golang-jwt/jwt/v5"
	"golang.org/x/crypto/bcrypt"
)

func Login(c *fiber.Ctx) error {
	var payload struct {
		Username string `json:"username"`
		Password string `json:"password"`
	}
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"message": "Invalid request"})
	}

	var hashed string
	var userID int
	err := db.Conn.QueryRow(context.Background(),
		"SELECT id, password FROM auth.users WHERE username=$1",
		payload.Username,
	).Scan(&userID, &hashed)
	if err != nil {
		return c.Status(401).JSON(fiber.Map{"message": "Unauthorized"})
	}

	err = bcrypt.CompareHashAndPassword([]byte(hashed), []byte(payload.Password))
	if err != nil {
		return c.Status(401).JSON(fiber.Map{"message": "Invalid credentials"})
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"user_id": userID,
		"role":    "admin",
		"exp":     time.Now().Add(time.Hour * 2).Unix(),
	})
	secret := os.Getenv("JWT_SECRET")
	signed, _ := token.SignedString([]byte(secret))

	return c.JSON(fiber.Map{
		"token": signed,
	})
}

func Refresh(c *fiber.Ctx) error {
	return c.JSON(fiber.Map{"message": "Refresh endpoint belum diimplementasi"})
}
