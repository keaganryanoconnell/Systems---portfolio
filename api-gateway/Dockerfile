FROM rust:1.85-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY . .
RUN cargo build --release -p api-gateway

FROM alpine:3.20
RUN apk add --no-cache ca-certificates
COPY --from=builder /build/target/release/api-gateway /app
EXPOSE 8080 443
ENTRYPOINT ["/app"]
