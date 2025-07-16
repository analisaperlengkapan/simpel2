package scheduler

import (
	"context"
	"log"
	"os"
	"os/signal"
	"syscall"

	"layanan-integrasi/internal/integrasi/monsakti"
	"layanan-integrasi/internal/integrasi/mysimkari"
	"layanan-integrasi/internal/integrasi/siman"

	"github.com/robfig/cron/v3"
)

// Start initializes and runs scheduled synchronization jobs
func Start() {
	c := cron.New(
		cron.WithSeconds(), // Optional: allow per-second granularity if needed
	)

	// Sinkronisasi harian: MySimkari & MONSAKTI
	_, err := c.AddFunc("@daily", func() {
		log.Println("[CRON] Sinkronisasi harian dimulai: MySimkari & MONSAKTI")
		ctx := context.Background()

		if err := mysimkari.Sync(ctx); err != nil {
			log.Printf("[MySimkari] Gagal sinkronisasi: %v\n", err)
		}
		if err := monsakti.Sync(ctx); err != nil {
			log.Printf("[Monsakti] Gagal sinkronisasi: %v\n", err)
		}
	})
	if err != nil {
		log.Println("[CRON] Gagal menambahkan job harian:", err)
	}

	// Sinkronisasi mingguan: SIMAN
	_, err = c.AddFunc("@weekly", func() {
		log.Println("[CRON] Sinkronisasi mingguan dimulai: SIMAN")
		ctx := context.Background()

		if err := siman.Sync(ctx); err != nil {
			log.Printf("[Siman] Gagal sinkronisasi: %v\n", err)
		}
	})
	if err != nil {
		log.Println("[CRON] Gagal menambahkan job mingguan:", err)
	}

	c.Start()
	log.Println("[CRON] Scheduler dimulai...")

	// Graceful shutdown agar cron berhenti saat SIGINT/SIGTERM
	sigs := make(chan os.Signal, 1)
	signal.Notify(sigs, os.Interrupt, syscall.SIGTERM)
	<-sigs

	log.Println("[CRON] Scheduler dihentikan.")
	c.Stop()
}
