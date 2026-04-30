#!/bin/bash
# PostgreSQL init script for docker-compose local development.
#
# Mounted into the postgres container at
# /docker-entrypoint-initdb.d/init-databases.sh — the official postgres
# image runs every *.sh / *.sql file in that directory exactly once, on
# first startup (i.e. when the data volume is empty). On subsequent
# starts the databases already exist and this script is not re-run.
#
# The Rust services and Laravel v1 each expect their own database; the
# default `postgres` database is not sufficient. Without this script,
# every dependent service fails at startup with
# "database \"dbsimpelvN\" does not exist".
#
# IMPORTANT — file mode: this script SHOULD be committed with mode 0755
# (executable). The official postgres entrypoint checks `[ -x "$f" ]`
# for files in /docker-entrypoint-initdb.d/: executable scripts are run
# as a subprocess, non-executable ones are `source`d into the
# entrypoint's own shell. To keep this script safe under either path,
# the body is wrapped in a function invoked in a subshell — that way
# `set -euo pipefail` only applies inside the subshell and does not
# leak `nounset` (etc.) into the parent entrypoint shell, which would
# otherwise crash postgres init the first time it touched an unset
# variable. Set the executable bit with:
#   git update-index --chmod=+x scripts/init-postgres-databases.sh

simpel_init_databases() (
  set -euo pipefail

  DATABASES=(
    dbsimpelv1
    dbsimpelv2
    dbsecretion
    dbintegration
  )

  for db in "${DATABASES[@]}"; do
    echo "Creating database '$db' if it does not exist..."
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
      SELECT 'CREATE DATABASE "$db"'
      WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '$db')\gexec
EOSQL
  done

  echo "All required databases have been created."
)

simpel_init_databases
