# 🔐 AGENTS.md — lib-crypto

> Crate ini berisi **primitif kriptografi WASM-compatible** untuk SIMPEL.

## 📍 Peran dalam Arsitektur

```
lib-core (types) ◄──depends── lib-crypto (crypto primitives)
                              │
                              ├── Shamir Secret Sharing (VSS)
                              ├── Password Hashing (Argon2id, bcrypt)
                              ├── AES-GCM Encryption (optional)
                              └── KDF (HKDF, PBKDF2) (optional)
```

## 📦 Modul & Feature Flags

| Feature | Modul | Dependensi |
|---------|-------|------------|
| (default) | `shamir`, `password` | curve25519-dalek, argon2, bcrypt |
| `encryption` | `aes`, `kdf` | aes-gcm, chacha20poly1305, hkdf, pbkdf2 |

## 📏 Aturan

1. **WASM-compatible**: Tidak boleh ada dependensi OS-level atau async runtime.
2. Gunakan `zeroize` untuk semua secret values.
3. Password hashing default: **Argon2id** (bukan bcrypt).
4. Gunakan `subtle::ConstantTimeEq` untuk perbandingan secret.
5. Shamir shares harus menggunakan `curve25519-dalek` untuk commitments.

## 🔑 Penggunaan

### Password Hashing
```rust
use lib_crypto::password::{hash_password, verify_password};
let hash = hash_password("my_password")?;
let valid = verify_password(&hash, "my_password")?;
```

### Shamir Secret Sharing
```rust
use lib_crypto::shamir::{ShamirConfig, generate_shares_with_commitments};
let config = ShamirConfig::new(3, 5)?; // threshold=3, total=5
let (shares, commitments) = generate_shares_with_commitments(&secret, &config)?;
```

### AES-GCM (requires `encryption` feature)
```rust
use lib_crypto::aes::{aes_gcm_encrypt, aes_gcm_decrypt, generate_key};
let key = generate_key();
let ciphertext = aes_gcm_encrypt(&key, &plaintext)?;
let plaintext = aes_gcm_decrypt(&key, &ciphertext)?;
```

## ⚠️ Pitfall

- `cfg(feature = "vss")` dalam `shamir.rs` belum didefinisikan sebagai feature — itu adalah dead code. Tambahkan feature `vss` jika diperlukan.
- Consumer harus enable feature `encryption` untuk menggunakan `aes` dan `kdf` modules.
