# Deploying a Full-Stack SSR App

The most popular way for people to deploy full-stack apps built with `cargo-leptos` is to use a cloud hosting service that supports deployment via a Docker build. Here’s a sample `Dockerfile`, which is based on the one we use to deploy the Leptos website.

```dockerfile
# Get started with a build env with Rust nightly
FROM rustlang/rust:nightly-bullseye as builder

# If you’re using stable, use this instead
# FROM rust:1.74-bullseye as builder

# Install cargo-binstall, which makes it easier to install other
# cargo extensions like cargo-leptos
RUN wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN tar -xvf cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN cp cargo-binstall /usr/local/cargo/bin

# Install cargo-leptos
RUN cargo binstall cargo-leptos -y

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

# Make an /app dir, which everything will eventually live in
RUN mkdir -p /app
WORKDIR /app
COPY . .

# Build the app
RUN cargo leptos build --release -vv

FROM rustlang/rust:nightly-bullseye as runner
# Copy the server binary to the /app directory
COPY --from=builder /app/target/release/leptos-start /app/
# /target/site contains our JS/WASM/CSS, etc.
COPY --from=builder /app/target/site /app/site
# Copy Cargo.toml if it’s needed at runtime
COPY --from=builder /app/Cargo.toml /app/
WORKDIR /app

# Set any required env variables and
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"
EXPOSE 8080
# Run the server
CMD ["/app/leptos_start"]
```

> Read more: [`gnu` and `musl` build files for Leptos apps](https://github.com/leptos-rs/leptos/issues/1152#issuecomment-1634916088).


## Deploy to Fly.io


## Deploy to Shuttle.rs

[Leptos Axum Starter Template for Shuttle.rs](https://github.com/Rust-WASI-WASM/shuttle-leptos-axum)

## Deploy to AWS Lambda

[Leptos Axum Starter Template for AWS Lambda](https://github.com/leptos-rs/start-aws)