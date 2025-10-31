# Gerbang - API Gateway untuk SIMPelv2

## Deskripsi

**Gerbang** adalah API Gateway yang menggunakan [Envoy Proxy](https://www.envoyproxy.io/) untuk sistem SIMPelv2. Komponen ini berfungsi sebagai entry point utama untuk semua permintaan API, menyediakan berbagai fitur seperti routing, load balancing, circuit breaking, rate limiting, dan keamanan.

## Fitur Utama

- **🔄 Routing & Load Balancing**: Mengarahkan permintaan ke berbagai layanan backend
- **🛡️ Circuit Breaker**: Melindungi sistem dari kegagalan beruntun
- **⏱️ Rate Limiting**: Mengontrol jumlah permintaan per detik
- **🔒 CORS**: Mengelola Cross-Origin Resource Sharing
- **🔐 Security Headers**: Menambahkan header keamanan penting
- **📊 Monitoring**: Admin interface untuk monitoring dan debugging
- **🔄 Retry Logic**: Mekanisme retry otomatis untuk permintaan yang gagal

## Arsitektur

```
Client Request → Gerbang (Port 8080) → Route to Clusters:
                                       ├── gerbang (Port 8080)
                                       ├── src (Port 7000)
                                       └── db (Port 8080)
```

## Konfigurasi Routes

### Routes Tersedia

1. **Default Route** (`/`): Mengarah ke cluster `gerbang`
2. **Source Route** (`/api/src`): Mengarah ke cluster `src`
3. **Database Route** (`/api/db`): Mengarah ke cluster `db`

### Konfigurasi Cluster

| Cluster | Port | Circuit Breaker | Max Requests | Max Retries |
|---------|------|-----------------|--------------|-------------|
| gerbang | 8080 | ✅ | 1000 | 3 |
| src     | 7000 | ✅ | 500  | 2 |
| db      | 8080 | ✅ | 500  | 2 |

## Instalasi & Setup

### Prerequisites

- Docker dan Docker Compose
- Akses ke sistem SIMPelv2

### Build dan Run

```bash
# Build image
docker build -t simpel-gerbang:latest ./infra/gerbang

# Run container
docker run -d \
  --name simpel-gerbang \
  -p 8080:8080 \
  -p 9901:9901 \
  simpel-gerbang:latest
```

### Menggunakan Docker Compose

```yaml
version: '3.8'
services:
  gerbang:
    build:
      context: ./infra/gerbang
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
      - "9901:9901"
    environment:
      - ENVOY_PORT=8080
      - ENVOY_ADMIN_PORT=9901
      - LOG_LEVEL=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9901/stats"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

## Environment Variables

| Variable | Default | Deskripsi |
|----------|---------|-----------|
| `ENVOY_PORT` | `8080` | Port utama untuk API Gateway |
| `ENVOY_ADMIN_PORT` | `9901` | Port untuk admin interface |
| `LOG_LEVEL` | `info` | Level logging (trace, debug, info, warn, error) |

## Health Check

Gerbang menyediakan endpoint health check pada:
- **Path**: `/health`
- **Port**: `8080`
- **Admin Stats**: `http://localhost:9901/stats`

## Monitoring & Debugging

### Admin Interface

Akses admin interface Envoy di `http://localhost:9901` untuk:
- Statistics dan metrics
- Configuration dump
- Health status
- Request routing info

### Logging

Log dapat dilihat dengan:
```bash
# Melihat log container
docker logs simpel-gerbang

# Log real-time
docker logs -f simpel-gerbang
```

## Security Features

### CORS Configuration

```yaml
allow_origin: ["*"]
allow_methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
allow_headers: ["*"]
max_age: "86400"
```

### Security Headers

Otomatis menambahkan header:
- `Content-Security-Policy: default-src 'self'`
- `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload`
- `X-Frame-Options: DENY`
- `X-Content-Type-Options: nosniff`
- `Referrer-Policy: no-referrer`
- `Permissions-Policy: geolocation=(), microphone=()`

## Dependencies

Gerbang bergantung pada layanan:
- **layanan-keamanan**: Untuk autentikasi dan autorisasi
- **layanan-integrasi**: Untuk integrasi dengan sistem eksternal

## Troubleshooting

### Common Issues

1. **Port sudah digunakan**
   ```bash
   # Cek proses yang menggunakan port
   lsof -i :8080
   # Atau gunakan port berbeda
   docker run -p 8081:8080 ...
   ```

2. **Health check gagal**
   - Pastikan semua dependency services berjalan
   - Cek konfigurasi cluster di `envoy.yaml`
   - Lihat log untuk error details

3. **High latency atau timeout**
   - Periksa konfigurasi `timeout` di route
   - Cek circuit breaker thresholds
   - Monitor resource usage

### Debug Mode

Untuk debugging yang lebih detail:
```bash
docker run -e LOG_LEVEL=debug simpel-gerbang:latest
```

## API Documentation

### Base URL
```
http://localhost:8080
```

### Endpoints

- **Health Check**: `GET /health`
- **Admin Interface**: `GET http://localhost:9901/stats`
- **Configuration**: `GET http://localhost:9901/config_dump`

## Contributing

1. Fork repository
2. Buat feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit changes (`git commit -m 'Add some AmazingFeature'`)
4. Push ke branch (`git push origin feature/AmazingFeature`)
5. Buat Pull Request

## License

Bagian dari proyek SIMPelv2. Lihat LICENSE utama untuk detail lebih lanjut.

## Support

Untuk pertanyaan atau masalah:
- Buat issue di repository
- Hubungi tim development
- Dokumentasi: [SIMPelv2 Docs](../../docs/)

---

*Gerbang v1.0.0 - API Gateway untuk SIMPelv2*
