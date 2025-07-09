package middleware

import (
	"bytes"
	"encoding/json"
	"log"
	"net/http"
	"os"
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

func AuditLogger() fiber.Handler {
	return func(c *fiber.Ctx) error {
		start := time.Now()
		err := c.Next()

		// Data untuk dikirim ke layanan-audit
		audit := AuditLog{
			UserID:    c.Locals("user_id"),
			Method:    c.Method(),
			Path:      c.Path(),
			Timestamp: start,
			IP:        c.IP(),
		}

		go sendAuditLog(audit)
		return err
	}
}

func sendAuditLog(logData AuditLog) {
	payload, _ := json.Marshal(logData)
	url := os.Getenv("AUDIT_URL")
	if url == "" {
		log.Println("AUDIT_URL not set")
		return
	}

	req, _ := http.NewRequest("POST", url, bytes.NewBuffer(payload))
	req.Header.Set("Content-Type", "application/json")

	client := http.Client{Timeout: 5 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		log.Printf("Failed to send audit log: %v", err)
		return
	}
	defer resp.Body.Close()
}
