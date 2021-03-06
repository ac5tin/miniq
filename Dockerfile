# Builder
FROM rustlang/rust:nightly as builder
COPY . /app
WORKDIR /app
RUN cargo build --release


# Final image
FROM debian:bookworm
COPY --from=builder /app/target/release/miniq /bin/
CMD miniq