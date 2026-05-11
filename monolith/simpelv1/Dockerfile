# Multi-stage build for SIMPEL v1 (Laravel) - PORT 8080
FROM php:8.2-fpm-alpine AS base

WORKDIR /app

# Install system dependencies + headers untuk PHP extensions.
# CATATAN: JANGAN install `composer` via apk — apk composer dipasangkan ke PHP
# system Alpine (PHP 8.4 di Alpine 3.21+), bukan PHP 8.2 dari base image.
# Composer di-install via official installer di step terpisah.
RUN apk add --no-cache \
    postgresql-client \
    postgresql-dev \
    libpq-dev \
    nginx \
    ca-certificates \
    curl \
    git \
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

# Install Composer 2 via official installer untuk PHP 8.2 (bukan apk composer
# yang dipasangkan ke PHP system Alpine).
RUN curl -fsSL https://getcomposer.org/installer | php -- \
        --install-dir=/usr/local/bin --filename=composer \
    && composer --version

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

# Create necessary directories.
# IMPORTANT: chown ke www-data (uid 82 di Alpine) bukan nobody — PHP-FPM
# worker run as www-data 82 dan butuh read access ke /app. Sebelumnya
# nobody:nobody (uid 65534) bikin PHP-FPM open() return EPERM walau
# mode 644 (Alpine package nobody might have weird attr).
RUN mkdir -p storage/logs storage/framework/sessions storage/framework/views storage/framework/cache \
              /var/lib/nginx/logs /var/lib/nginx/tmp/client_body \
              /var/lib/nginx/tmp/proxy /var/lib/nginx/tmp/fastcgi \
              /var/lib/nginx/tmp/uwsgi /var/lib/nginx/tmp/scgi \
              /run/nginx /var/cache/nginx \
    && chown -R www-data:www-data /app /var/lib/nginx /run/nginx /var/cache/nginx \
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
