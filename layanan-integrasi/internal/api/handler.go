// layanan-integrasi/internal/api/handler.go

package api

import (
	"net/http"

	"simpelv2/layanan-integrasi/internal/integrasi/mysimkari"

	"github.com/gin-gonic/gin"
)

func StartServer(cfg *mysimkari.Config) {
	gin.SetMode(cfg.Server.Mode)
	r := gin.Default()

	r.GET("/health", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	r.POST("/api/integrasi/mysimkari", func(c *gin.Context) {
		go mysimkari.SyncAllSatker(cfg)
		c.JSON(http.StatusOK, gin.H{"message": "Sinkronisasi dimulai"})
	})

	if err := r.Runf(":%d", cfg.Server.Port); err != nil {
		panic(err)
	}
}
