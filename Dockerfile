FROM rust:1.91.1-alpine AS builder
RUN apk add pkgconfig openssl openssl-dev openssl-libs-static musl-dev
COPY . /app
WORKDIR /app
RUN cargo b -r

FROM alpine:3.20.3
RUN mkdir /app
COPY --from=builder /app/target/release/orange-whale /app
WORKDIR /app
ENTRYPOINT ["./orange-whale"]
