FROM golang:1.23

WORKDIR /app

COPY backend/go.mod backend/go.sum ./
RUN go mod download

COPY backend/ ./
RUN go install github.com/gobuffalo/cli/cmd/buffalo@latest

EXPOSE 3000

CMD ["buffalo", "dev"]
