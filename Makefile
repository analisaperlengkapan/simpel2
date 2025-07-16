.PHONY: all up down build logs ps restart \
	generate-k8s validate-k8s regenerate-k8s \
	build-images export-images import-images build-and-import \
	deploy-dev deploy-staging deploy-prod \
	check-ns apply-base apply-ingress apply-secrets apply-monitoring \
	check-status restart-failed logs-k8s logs-all destroy-k8s clean-tar \
	seal-secret simulate-pod-run check-microk8s-access check-deps

# ====== CONFIG ======
COMPOSE_FILE ?= docker-compose.secure.yml
K8S_DIR = k8s
K8S_BASE = $(K8S_DIR)/base
K8S_OVERLAY_DEV = $(K8S_DIR)/overlays/dev
K8S_OVERLAY_STAGING = $(K8S_DIR)/overlays/staging
K8S_OVERLAY_PROD = $(K8S_DIR)/overlays/prod
K8S_MONITORING = $(K8S_DIR)/monitoring
K8S_SECRETS = $(K8S_DIR)/secrets
K8S_INGRESS = $(K8S_DIR)/ingress
NAMESPACE = simpelv2
KUBECTL = microk8s kubectl
IMAGES = gerbang antarmuka layanan-audit layanan-keamanan layanan-integrasi

ENV ?= dev
VERSION ?= v1.0.0
ifeq ($(ENV),prod)
TAG := $(VERSION)
else
TAG := $(ENV)
endif

# ====== DOCKER COMPOSE ======
up:
	docker compose -f $(COMPOSE_FILE) up -d

down:
	docker compose -f $(COMPOSE_FILE) down

build:
	docker compose -f $(COMPOSE_FILE) build

logs:
	docker compose -f $(COMPOSE_FILE) logs -f --tail=100

ps:
	docker compose -f $(COMPOSE_FILE) ps

restart: down up

# ====== BUILD & IMPORT IMAGE ======
build-images:
	@echo "🔨 Membangun image dengan tag :$(TAG)..."
	@for service in $(IMAGES); do \
		docker build -t simpelv2/$$service:$(TAG) ./$$service || exit 1; \
	done

export-images:
	@echo "📦 Mengekspor image ke file TAR..."
	@for service in $(IMAGES); do \
		docker save -o ./$$service-$(TAG).tar simpelv2/$$service:$(TAG) || exit 1; \
	done

import-images:
	@echo "🚛 Mengimpor ke MicroK8s..."
	@for service in $(IMAGES); do \
		microk8s ctr image import ./$$service-$(TAG).tar || exit 1; \
	done

clean-tar:
	@rm -f ./*-$(TAG).tar

build-and-import: check-microk8s-access build-images export-images import-images clean-tar

# ====== K8S YAML ======
generate-k8s:
	@echo "🔧 Menjalankan generate_k8s.py untuk env: $(ENV), tag: $(TAG)..."
	@ENV=$(ENV) TAG=$(TAG) python3 scripts/generate_k8s.py

validate-k8s:
	@echo "✅ Validasi semua file YAML..."
	kubeval --strict --ignore-missing-schemas $(K8S_BASE)/*.yaml || (echo "❌ Validasi gagal."; exit 1)

regenerate-k8s: generate-k8s validate-k8s

# ====== DEPLOY PER ENV ======
deploy-dev: ENV=dev
deploy-dev: pre-deploy-check build-and-import regenerate-k8s apply-base apply-ingress apply-secrets apply-monitoring
	$(KUBECTL) apply -k $(K8S_OVERLAY_DEV)
	$(MAKE) check-status
	$(MAKE) restart-failed
	@echo "✅ SIMPelv2 berhasil dideploy ke DEV."

deploy-staging: ENV=staging
deploy-staging: pre-deploy-check build-and-import regenerate-k8s apply-base apply-ingress apply-secrets apply-monitoring
	$(KUBECTL) apply -k $(K8S_OVERLAY_STAGING)
	$(MAKE) check-status
	$(MAKE) restart-failed
	@echo "✅ SIMPelv2 berhasil dideploy ke STAGING."

deploy-prod: ENV=prod
deploy-prod: pre-deploy-check build-and-import regenerate-k8s apply-base apply-ingress apply-secrets apply-monitoring
	$(KUBECTL) apply -k $(K8S_OVERLAY_PROD)
	$(MAKE) check-status
	$(MAKE) restart-failed
	@echo "✅ SIMPelv2 berhasil dideploy ke PROD (versi $(VERSION))."

# ====== RESOURCE APPLY ======
check-ns:
	@echo "🔍 Memeriksa namespace '$(NAMESPACE)'..."
	@if ! $(KUBECTL) get ns $(NAMESPACE) > /dev/null 2>&1; then \
		echo "📁 Namespace belum ada. Membuat..."; \
		$(KUBECTL) apply -f $(K8S_BASE)/namespace.yaml; \
	else \
		echo "✅ Namespace '$(NAMESPACE)' tersedia."; \
	fi

apply-base:
	@echo "📦 Deploying base services..."
	$(KUBECTL) apply -f $(K8S_BASE)

apply-ingress:
	@echo "🌐 Deploying ingress rules..."
	$(KUBECTL) apply -f $(K8S_INGRESS)/ingress.yaml

apply-secrets:
	@echo "🔐 Deploying SealedSecrets..."
	@if [ ! -d $(K8S_SECRETS)/sealed ]; then \
		echo "❌ Direktori 'sealed' tidak ditemukan. Jalankan 'make generate-k8s' dulu."; \
		exit 1; \
	else \
		$(KUBECTL) apply -f $(K8S_SECRETS)/sealed; \
	fi

apply-monitoring:
	@echo "📊 Deploying monitoring stack..."
	@if [ -d $(K8S_MONITORING) ]; then \
		$(KUBECTL) apply -f $(K8S_MONITORING); \
	else \
		echo "⚠️  Tidak ada konfigurasi monitoring."; \
	fi

# ====== MONITORING STATUS ======
check-status:
	@echo "🔎 Mengecek status pod di namespace $(NAMESPACE)..."
	@$(KUBECTL) get pods -n $(NAMESPACE) -o wide

restart-failed:
	@echo "♻️  Merestart pods yang statusnya CrashLoopBackOff atau Error..."
	@$(KUBECTL) get pods -n $(NAMESPACE) --no-headers | awk '$$3 ~ /CrashLoopBackOff|Error/ {print $$1}' | while read pod; do \
		echo "🔁 Restarting pod: $$pod"; \
		$(KUBECTL) delete pod $$pod -n $(NAMESPACE); \
	done

logs-k8s:
	$(KUBECTL) logs -n $(NAMESPACE) -l app=gerbang --tail=100 -f

logs-all:
	@echo "📄 Menampilkan log semua pods (tail 50)..."
	@$(KUBECTL) get pods -n $(NAMESPACE) -o name | while read pod; do \
		echo "==> $$pod"; \
		$(KUBECTL) logs -n $(NAMESPACE) $$pod --tail=50 || true; \
	done

destroy-k8s:
	@echo "🧹 Menghapus semua resource di namespace $(NAMESPACE)..."
	$(KUBECTL) delete all --all -n $(NAMESPACE)

# ====== SEALED SECRET ======
seal-secret:
	@echo "🔐 Membuat sealed secret (kubeseal harus terinstall)..."
	@if [ -z "$(NAME)" ] || [ ! -f "$(FILE)" ]; then \
		echo "❌ Harap set variabel NAME dan FILE. Contoh: make seal-secret NAME=db FILE=.env"; \
	else \
		kubectl create secret generic $(NAME) --from-env-file=$(FILE) --dry-run=client -o yaml | \
		kubeseal --controller-namespace kube-system --format yaml > $(K8S_SECRETS)/sealed/$(NAME)-sealed.yaml && \
		echo "✅ Sealed secret $(NAME) berhasil dibuat."; \
	fi

# ====== VALIDASI DAN LINTING ======
validate-env:
	@echo "🔍 Memvalidasi file .env terhadap .env.example..."
	@python3 scripts/validate_env.py

lint-yaml:
	@echo "🧹 Melakukan linting semua file YAML..."
	@yamllint -c .yamllint.yaml $(K8S_DIR)

simulate-pod-run:
	@echo "🧪 Menyimulasikan semua image berjalan..."
	@for service in $(IMAGES); do \
		echo "🔍 Menjalankan test image: simpelv2/$$service:$(TAG)"; \
		docker run --rm simpelv2/$$service:$(TAG) true || (echo "❌ Gagal menjalankan $$service, periksa binary atau CMD."; exit 1); \
	done

pre-deploy-check: check-deps generate-k8s validate-env lint-yaml validate-k8s simulate-pod-run check-ns
	@echo "✅ Semua pre-deploy check lulus."

# ====== MICROK8S & DEPENDENCY CHECK ======
check-microk8s-access:
	@echo "🔐 Memeriksa apakah user '$(USER)' tergabung dalam grup 'microk8s'..."
	@if ! groups $(USER) | grep -q '\bmicrok8s\b'; then \
		echo "❌ User $(USER) belum tergabung di grup 'microk8s'. Akibatnya, perintah 'microk8s' akan minta sudo."; \
		echo "➡️  Solusi: sudo usermod -a -G microk8s $(USER) && newgrp microk8s"; \
		exit 1; \
	else \
		echo "✅ Akses microk8s OK tanpa sudo."; \
	fi

check-deps:
	@echo "🔍 Memeriksa dependency untuk SIMPelv2..."
	@which python3 > /dev/null || (echo "⚠️ Python3 tidak ditemukan, mencoba install..."; sudo apt install -y python3)
	@which docker > /dev/null || (echo "⚠️ Docker tidak ditemukan, mencoba install..."; sudo apt install -y docker.io)
	@which microk8s > /dev/null || (echo "⚠️ microk8s tidak ditemukan, mencoba install..."; sudo snap install microk8s --classic)
	@which kubeval > /dev/null || (echo "⚠️ kubeval tidak ditemukan, mencoba install..."; curl -s https://api.github.com/repos/instrumenta/kubeval/releases/latest | grep browser_download_url | grep linux-amd64 | cut -d '"' -f 4 | wget -qi - && chmod +x kubeval && sudo mv kubeval /usr/local/bin)
	@which yamllint > /dev/null || (echo "⚠️ yamllint tidak ditemukan, mencoba install..."; sudo apt install -y yamllint)
	@which kubeseal > /dev/null || (echo "⚠️ kubeseal tidak ditemukan, mencoba install..."; curl -sL https://github.com/bitnami-labs/sealed-secrets/releases/latest/download/kubeseal-linux-amd64 -o kubeseal && chmod +x kubeseal && sudo mv kubeseal /usr/local/bin)
	@test -f $(COMPOSE_FILE) || (echo "❌ File $(COMPOSE_FILE) tidak ditemukan"; exit 1)
	@test -f scripts/generate_k8s.py || (echo "❌ scripts/generate_k8s.py tidak ditemukan"; exit 1)
	@echo "✅ Semua dependency ditemukan atau berhasil di-install."
