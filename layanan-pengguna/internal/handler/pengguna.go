package handler

import (
	"context"
	"layanan-pengguna/internal/db"

	"github.com/gofiber/fiber/v2"
)

type Pengguna struct {
	ID       int    `json:"id"`
	Nama     string `json:"nama"`
	Username string `json:"username"`
	Satker   string `json:"satker"`
	Role     string `json:"role"`
}

func GetPengguna(c *fiber.Ctx) error {
	id := c.Params("id")
	var p Pengguna
	err := db.Conn.QueryRow(context.Background(),
		`SELECT id, nama, username, satker, role FROM pengguna.pengguna WHERE id=$1`, id,
	).Scan(&p.ID, &p.Nama, &p.Username, &p.Satker, &p.Role)
	if err != nil {
		return c.Status(404).JSON(fiber.Map{"message": "Pengguna tidak ditemukan"})
	}
	return c.JSON(p)
}

func ListPengguna(c *fiber.Ctx) error {
	rows, err := db.Conn.Query(context.Background(),
		`SELECT id, nama, username, satker, role FROM pengguna.pengguna`)
	if err != nil {
		return c.Status(500).SendString(err.Error())
	}
	defer rows.Close()

	var data []Pengguna
	for rows.Next() {
		var p Pengguna
		if err := rows.Scan(&p.ID, &p.Nama, &p.Username, &p.Satker, &p.Role); err != nil {
			continue
		}
		data = append(data, p)
	}

	return c.JSON(data)
}
