# Secreton Agent

**Secreton Agent** adalah aplikasi pendamping (helper/sidecar) untuk Secreton engine server. Agent ini bertugas melakukan auto-auth, perpanjangan token otomatis, rendering template file dari secret engine, serta sink token ke file, environment, atau menjalankan aplikasi lain dengan token dinamis. Agent ini sangat cocok untuk DevOps, deployment cloud-native, dan kebutuhan compliance/enterprise.

---

## Perbedaan Agent vs Server

| Komponen             | Secreton Server                  | Secreton Agent                       |
|---------------------|----------------------------------|--------------------------------------|
| **Fungsi utama**    | Server utama, API, storage, RBAC | Client/sidecar, auto-auth, template  |
| **Proses**          | Service utama, satu per cluster  | Banyak, satu per aplikasi/VM/Pod     |
| **Akses**           | Menyimpan & mengelola secrets    | Mengambil secrets, tidak menyimpan   |
| **Kegunaan**        | Backend, pusat keamanan          | Otomasi aplikasi, DevOps, CI/CD      |
| **Contoh deploy**   | VM, container, Kubernetes        | Sidecar, VM, container, pipeline     |
| **Auto-auth**       | Tidak (hanya API)                | Ya (userpass, approle, k8s)          |
| **Sink token**      | Tidak                            | Ya (file, env)                       |
| **Template**        | Tidak                            | Ya (render file dari secret)         |

**Singkatnya:**
- `secreton` = server utama, pusat API dan storage secret
- `secreton-agent` = client/sidecar untuk aplikasi, mengambil secret/token dari server

---

## Kegunaan Agent
- Otomatis login ke Secreton engine dan perpanjang token
- Render file konfigurasi dari secret engine ke file lokal (template)
- Sink token ke file atau environment variable
- Failover ke server backup jika server utama down
- Siap untuk DevOps dan cloud-native deployment

---

## Fitur Utama
- **Auto-auth**: userpass, approle, kubernetes
- **Token renewal**: otomatis setiap 5 menit (configurable)
- **Template rendering**: secret engine → file lokal
- **Token sink**: file, environment variable
- **Failover server**: multi-server support
- **Health endpoint**: HTTP `/healthz` untuk monitoring

---

## Contoh Konfigurasi (`agent.yaml`)
```yaml
server_url: "https://engine.example.com:8200"
server_urls:
  - "https://engine1.example.com:8200"
  - "https://engine2.example.com:8200"

auth_method: userpass
auth_config:
  username: "myuser"
  password: "mypassword"

templates:
  - source: "secret/data/myapp/config"
    dest: "/etc/myapp/config.json"
    permissions: "0600"

token_renewal_interval_secs: 300
template_interval_secs: 60

sink:
  types: ["file", "env"]
  file_path: "/var/run/secrets/engine-token"
  file_permissions: "0600"
  env_var: "VAULT_TOKEN"

health_port: 9900
```

---

## Cara Menjalankan
1. **Build**
   ```sh
   cargo build --release -p secreton-agent
   ```
2. **Jalankan**
   ```sh
   ./target/release/secreton-agent --config agent.yaml
   ```
3. **Health check**
   - Endpoint: `http://localhost:9900/healthz`
   - Response:
     ```json
     {
       "status": "ok",
       "token_valid": true
     }
     ```

---

## Best Practice
- Jalankan agent sebagai sidecar container di Kubernetes
- Gunakan token sink untuk aplikasi yang butuh token dinamis
- Gunakan health endpoint untuk monitoring otomatis
- Gunakan failover server untuk high-availability
- Set file permissions yang ketat untuk token dan config files
