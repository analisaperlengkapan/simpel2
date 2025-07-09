# Stage 1: Build React App
FROM node:20-alpine AS builder

WORKDIR /app
COPY . .

RUN npm install && npm run build

# Stage 2: Serve with NGINX
FROM nginx:stable-alpine AS production

# Copy custom nginx config (optional)
COPY ./nginx.conf /etc/nginx/conf.d/default.conf

# Copy build result from builder
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
