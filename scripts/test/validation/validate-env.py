#!/usr/bin/env python3
import os
from dotenv import dotenv_values

ENV_PATH = ".env"
EXAMPLE_PATH = ".env.example"

def read_env_file(filepath):
    try:
        with open(filepath, "r") as f:
            lines = f.readlines()
        return lines
    except Exception as e:
        print(f"❌ Gagal membaca {filepath}: {e}")
        exit(1)

def check_duplicates(lines, filename):
    seen = {}
    duplicates = []
    for line in lines:
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key = line.split("=")[0].strip()
        if key in seen:
            duplicates.append(key)
        seen[key] = True
    if duplicates:
        print(f"❌ Duplikat variabel ditemukan di {filename}: {', '.join(duplicates)}")
        return False
    return True

def validate_env():
    if not os.path.exists(ENV_PATH):
        print(f"❌ File {ENV_PATH} tidak ditemukan.")
        exit(1)

    if not os.path.exists(EXAMPLE_PATH):
        print(f"❌ File {EXAMPLE_PATH} tidak ditemukan.")
        exit(1)

    env_lines = read_env_file(ENV_PATH)
    example_lines = read_env_file(EXAMPLE_PATH)

    if not check_duplicates(env_lines, ENV_PATH):
        exit(1)

    env = dotenv_values(ENV_PATH)
    example = dotenv_values(EXAMPLE_PATH)

    missing = []
    for key in example:
        if key not in env or env[key] is None or str(env[key]).strip() == "":
            missing.append(key)

    if missing:
        print(f"❌ Variabel berikut hilang atau kosong di {ENV_PATH}:")
        for m in missing:
            print(" -", m)
        exit(1)

    for key, val in env.items():
        if isinstance(val, list) or isinstance(val, dict):
            print(f"❌ Format tidak valid untuk {key}: tidak boleh array atau object.")
            exit(1)

    print("✅ Semua variabel environment telah lengkap dan valid.")

if __name__ == "__main__":
    validate_env()
