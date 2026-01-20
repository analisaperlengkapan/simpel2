#!/bin/bash
# Wrapper script for docker-compose to avoid shell issues
exec python3 /usr/bin/docker-compose "$@"
