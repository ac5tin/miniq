# Builder
FROM rustlang/rust:nightly as builder
COPY . /app
WORKDIR /app
RUN cargo build --release


# Final image
FROM alpine:3.15
COPY --from=builder /app/target/release/miniq /usr/local/bin/miniq
CMD miniq