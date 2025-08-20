#!/bin/bash

set -e

echo "🛠️  Menginstall Docker Engine dan Compose V2..."

# 1. Hapus docker-compose v1 jika ada
sudo apt-get remove -y docker-compose || true

# 2. Hapus docker lama (jika ada)
sudo apt-get remove -y docker docker-engine docker.io containerd runc || true

# 3. Tambah GPG key Docker
sudo mkdir -p /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | \
  sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg

# 4. Tambah Docker repo
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
  https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# 5. Update dan install paket
sudo apt-get update
sudo apt-get install -y \
  docker-ce docker-ce-cli containerd.io \
  docker-buildx-plugin docker-compose-plugin

# 6. Aktifkan service docker (dan enable saat boot)
sudo systemctl enable docker
sudo systemctl start docker

# 7. Cek versi
docker --version
docker compose version

echo "✅ Docker dan Compose V2 berhasil diinstal."
