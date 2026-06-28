FROM nginx:1.27-alpine

# Remove default nginx config
RUN rm -rf /etc/nginx/conf.d/default.conf

# Build context is the repo root (see release.yml / ci.yml docker matrix:
# context "." for every image), so COPY paths are repo-root-relative.
COPY monolith/APP-2026/index.html /usr/share/nginx/html/index.html
COPY monolith/APP-2026/index2.html /usr/share/nginx/html/index2.html
COPY monolith/APP-2026/anbut.jpg /usr/share/nginx/html/anbut.jpg
COPY monolith/APP-2026/favicon.ico /usr/share/nginx/html/favicon.ico

# Custom nginx config for serving under subpath
COPY monolith/APP-2026/nginx.conf /etc/nginx/nginx.conf

# Use non-root user
RUN chown -R nginx:nginx /usr/share/nginx/html && \
    chmod -R 755 /usr/share/nginx/html && \
    chown -R nginx:nginx /var/cache/nginx && \
    chown -R nginx:nginx /var/log/nginx && \
    touch /var/run/nginx.pid && \
    chown -R nginx:nginx /var/run/nginx.pid

USER nginx
EXPOSE 8080

CMD ["nginx", "-g", "daemon off;"]
