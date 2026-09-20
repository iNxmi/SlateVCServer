FROM rust:latest AS builder
WORKDIR /application
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /application/target/release/slate_vc_server /usr/local/bin/
EXPOSE 8000
CMD ["slate_vc_server"]