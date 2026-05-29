FROM rust:1.85-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY . .
RUN cargo build --release -p raft-kv

FROM alpine:3.20
RUN apk add --no-cache ca-certificates
COPY --from=builder /build/target/release/raft-kv /app
EXPOSE 9000 9001 9002
ENTRYPOINT ["/app"]
