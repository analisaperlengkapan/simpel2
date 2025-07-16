FROM golang:1.23-alpine AS builder

WORKDIR /app
COPY . .

# Ubah nama output binary menjadi "main" agar seragam
RUN go build -o main ./cmd/main.go

FROM alpine:latest

WORKDIR /app
COPY --from=builder /app/main .

# Salin seluruh folder config (opsional lebih fleksibel)
COPY config/ ./config/

# Gunakan nama "main" agar sesuai dengan layanan lain
CMD ["./main"]
