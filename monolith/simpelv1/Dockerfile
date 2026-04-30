# Multi-stage build for SIMPEL v1 (Laravel) - PORT 8080
FROM php:8.2-fpm-alpine AS base

WORKDIR /app

# Install system dependencies
RUN apk add --no-cache \
    postgresql-client \
    postgresql-dev \
    libpq-dev \
    nginx \
    ca-certificates \
    curl \
    git \
    composer

# Install PHP extensions
RUN docker-php-ext-install pdo pdo_pgsql

# Install extensions needed for gRPC (optional, install protobuf-php via composer)
RUN apk add --no-cache --virtual .build-deps \
    autoconf \
    g++ \
    make \
    && apk add --no-cache protobuf protobuf-dev

# Copy composer files
COPY monolith/simpelv1/composer.json composer.json
COPY monolith/simpelv1/composer.lock composer.lock

# Install dependencies
RUN composer install --no-dev --no-scripts --no-autoloader

# Copy application code
COPY monolith/simpelv1 .

# Complete autoloader
RUN composer dump-autoload --no-scripts

# Generate optimized config
RUN composer run-script post-install-cmd || true

# Create necessary directories
RUN mkdir -p storage/logs storage/framework/sessions storage/framework/views storage/framework/cache \
    && chown -R nobody:nobody . \
    && chmod -R 755 storage

# Copy nginx config
COPY monolith/simpelv1/nginx.conf /etc/nginx/nginx.conf

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Expose port
EXPOSE 8080

# Set environment
ENV APP_DEBUG=false
ENV APP_ENV=production

# Run nginx
CMD ["nginx", "-g", "daemon off;"]
