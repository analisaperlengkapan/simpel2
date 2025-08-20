package handler

import (
	"context"
	"layanan-audit/internal/db"
	"time"

	"github.com/gofiber/fiber/v2"
)

type AuditLog struct {
	UserID    interface{} `json:"user_id"`
	Method    string      `json:"method"`
	Path      string      `json:"path"`
	Timestamp time.Time   `json:"timestamp"`
	IP        string      `json:"ip"`
}

func LogAudit(c *fiber.Ctx) error {
	var payload AuditLog
	if err := c.BodyParser(&payload); err != nil {
		return c.SendStatus(400)
	}

	_, err := db.Conn.Exec(context.Background(),
		"INSERT INTO audit.logs (user_id, method, path, timestamp, ip) VALUES ($1, $2, $3, $4, $5)",
		payload.UserID, payload.Method, payload.Path, payload.Timestamp, payload.IP,
	)
	if err != nil {
		return c.SendStatus(500)
	}
	return c.SendStatus(201)
}
