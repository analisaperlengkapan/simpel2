# Multi-stage build for SIMPEL v1 (Laravel) - PORT 8080
FROM php:8.2-fpm-alpine AS base

WORKDIR /app

# Install system dependencies + headers untuk PHP extensions
RUN apk add --no-cache \
    postgresql-client \
    postgresql-dev \
    libpq-dev \
    nginx \
    ca-certificates \
    curl \
    git \
    composer \
    # GD deps
    libpng-dev \
    libjpeg-turbo-dev \
    freetype-dev \
    # XML/DOM deps
    libxml2-dev \
    # Sodium
    libsodium-dev \
    # zip extension (untuk maatwebsite/excel)
    libzip-dev \
    # icu untuk intl (locale)
    icu-dev \
    # oniguruma untuk mbstring
    oniguruma-dev

# Install PHP extensions yang dibutuhkan Laravel framework + dependencies
# (session, tokenizer, ctype, json sudah built-in PHP 8.2)
RUN docker-php-ext-configure gd --with-freetype --with-jpeg \
    && docker-php-ext-install \
        pdo \
        pdo_pgsql \
        gd \
        dom \
        fileinfo \
        sodium \
        bcmath \
        mbstring \
        intl \
        zip \
        opcache

# Build dependencies for compiling PHP/protobuf extensions. Installed
# under a virtual package so they can be removed after composer install
# to keep the runtime image small and reduce attack surface.
RUN apk add --no-cache --virtual .build-deps \
    autoconf \
    g++ \
    make \
    protobuf-dev \
    && apk add --no-cache protobuf

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

# Remove build toolchain — the runtime only needs the protobuf runtime
# library, not autoconf/g++/make/protobuf-dev. Cuts image size and
# removes compilers from the production image.
RUN apk del .build-deps

# Create necessary directories
RUN mkdir -p storage/logs storage/framework/sessions storage/framework/views storage/framework/cache \
    && chown -R nobody:nobody . \
    && chmod -R 755 storage

# Copy nginx config
COPY monolith/simpelv1/nginx.conf /etc/nginx/nginx.conf

# Health check — uses Laravel's /health route so the full nginx+PHP-FPM
# stack is exercised, not just nginx.
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Expose port
EXPOSE 8080

# Set environment
ENV APP_DEBUG=false
ENV APP_ENV=production

# Run both PHP-FPM (background) and nginx (foreground).
# nginx.conf proxies PHP requests to upstream php_fpm at 127.0.0.1:9000,
# so PHP-FPM must be running for the app to serve any .php route.
CMD ["sh", "-c", "php-fpm -D && nginx -g 'daemon off;'"]
