FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /build
COPY . .

RUN cargo build --release -p compute-orchestrator --target x86_64-unknown-linux-musl

FROM scratch

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/compute-orchestrator /app
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

EXPOSE 9000
EXPOSE 7946/udp

ENTRYPOINT ["/app"]

CMD ["leader", "--bind", "0.0.0.0:9000"]
