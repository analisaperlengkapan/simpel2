//gerbang/internal/proxy/router.go

package proxy

import (
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/proxy"
)

func SetupRoutes(app *fiber.App) {
	// Routing semua prefix ke layanan terkait
	// AUTH
	app.All("/api/auth/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-kemanan:3000"},
	}))

	app.All("/auth/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-keamanan:3000"},
	}))

	// ASET
	app.All("/api/aset/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-aset:3000"},
	}))

	app.All("/aset/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-aset:3000"},
	}))

	// ROADMAP
	app.All("/api/roadmap/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-roadmap:3000"},
	}))

	app.All("/roadmap/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-roadmap:3000"},
	}))

	// PEMAKAIAN
	app.All("/api/pemakaian/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-pemakaian:3000"},
	}))

	app.All("/pemakaian/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-pemakaian:3000"},
	}))

	// HIBAH
	app.All("/api/hibah/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-hibah:3000"},
	}))

	app.All("/hibah/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-hibah:3000"},
	}))

	// PENGALIHAN
	app.All("/api/pengalihan/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-pengalihan:3000"},
	}))

	app.All("/pengalihan/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-pengalihan:3000"},
	}))

	// STANDAR
	app.All("/api/standar/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-standar:3000"},
	}))

	app.All("/standar/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-standar:3000"},
	}))

	// REFERENSI
	app.All("/api/referensi/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-referensi:3000"},
	}))

	app.All("/referensi/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-referensi:3000"},
	}))

	// UPLOAD
	app.All("/api/upload/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-upload:3000"},
	}))

	app.All("/upload/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-upload:3000"},
	}))

	// LAPORAN
	app.All("/api/laporan/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-laporan:3000"},
	}))

	app.All("/laporan/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-laporan:3000"},
	}))

	// INTEGRASI
	app.All("/api/integrasi/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-integrasi:3000"},
	}))

	app.All("/integrasi/*", proxy.Balancer(proxy.Config{
		Servers: []string{"http://layanan-integrasi:3000"},
	}))
}
