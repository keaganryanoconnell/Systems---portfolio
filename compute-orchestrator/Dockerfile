FROM rust:1.85-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev

RUN addgroup -g 1000 appuser && adduser -D -u 1000 -G appuser appuser

WORKDIR /build
COPY . .

RUN cargo build --release -p compute-orchestrator --target x86_64-unknown-linux-musl

FROM scratch

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/compute-orchestrator /app
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

USER 1000:1000

EXPOSE 9000
EXPOSE 7946/udp

HEALTHCHECK --interval=30s --timeout=3s --retries=3 CMD ["/app", "status"] || exit 1

ENTRYPOINT ["/app"]

CMD ["leader", "--bind", "0.0.0.0:9000"]
