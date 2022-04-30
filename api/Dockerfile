FROM rust:1-alpine3.15
# This is important, see https://github.com/rust-lang/docker-rust/issues/85
ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /app
COPY ./ /app

RUN cargo build --release
RUN strip target/release/api

# use a plain alpine image, the alpine version needs to match the builder
FROM alpine:3.15

RUN apk add --no-cache libgcc
COPY --from=0 /app/target/release/api .

ENTRYPOINT ["/api"]
