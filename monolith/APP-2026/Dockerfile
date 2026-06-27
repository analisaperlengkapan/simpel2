FROM nginx:1.27-alpine

# Remove default nginx config
RUN rm -rf /etc/nginx/conf.d/default.conf

# Copy static files
COPY index.html /usr/share/nginx/html/index.html
COPY anbut.jpg /usr/share/nginx/html/anbut.jpg
COPY favicon.ico /usr/share/nginx/html/favicon.ico

# Custom nginx config for serving under subpath
COPY nginx.conf /etc/nginx/nginx.conf

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
