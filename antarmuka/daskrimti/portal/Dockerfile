# Multi-stage build for Portal Microfrontend (Workspace-aware)
FROM rustlang/rust:nightly as builder

# Install WASM toolchain and trunk
RUN rustup target add wasm32-unknown-unknown
RUN wget -qO- https://github.com/trunk-rs/trunk/releases/download/v0.18.4/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C /usr/local/bin
RUN cargo install wasm-bindgen-cli --version 0.2.106

WORKDIR /app

# Copy entire workspace
COPY . .

# Build the portal microfrontend
WORKDIR /app/antarmuka/daskrimti/portal

# Set environment variables
ENV WASM_BINDGEN_WEAKREF=1
ENV RUST_BACKTRACE=1

# Build with trunk
RUN trunk build --release --public-url "/" -v

# Stage 2: Nginx for serving WASM
FROM nginx:alpine

# Copy build output
COPY --from=builder /app/antarmuka/daskrimti/portal/dist /usr/share/nginx/html

# Custom nginx config for SPA
RUN echo 'server { \
    listen 80; \
    server_name _; \
    root /usr/share/nginx/html; \
    index index.html; \
    location / { \
    try_files $uri $uri/ /index.html; \
    } \
    gzip on; \
    gzip_types text/css application/javascript application/wasm; \
    }' > /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]

