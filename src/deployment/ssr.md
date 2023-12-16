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

# -- NB: update binary name from "leptos-start" to match your app name in Cargo.toml --
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

# -- NB: update binary name from "leptos_start" to match your app name in Cargo.toml --
# Run the server
CMD ["/app/leptos_start"]
```

> Read more: [`gnu` and `musl` build files for Leptos apps](https://github.com/leptos-rs/leptos/issues/1152#issuecomment-1634916088).





## Deploy to Fly.io

First, create a `Dockerfile` in the root of your application and fill it in with the suggested contents (above); make sure to update the binary names in the Dockerfile example
to the name of your own application, and make other adjustments as necessary.

Also, ensure you have the `flyctl` CLI tool installed, and have an account set up at [Fly.io](https://fly.io/). To install `flyctl` on MacOS, Linux, or Windows WSL, run:

```sh
curl -L https://fly.io/install.sh | sh
```

If you have issues, or for installing to other platforms [see the full instructions here](https://fly.io/docs/hands-on/install-flyctl/)

Then login to Fly.io

```sh
fly auth login
```

and manually launch your app using the command

```sh
fly launch
```

The `flyctl` CLI tool will walk you through the process of deploying your app to Fly.io.

<br/>

If you would prefer to use Github Actions to manage your deployments, you will need to create a new access token via the [Fly.io](https://fly.io/) web UI.

Go to "Account" > "Access Tokens" and create a token named something like "github_actions", then add the token to your Github repo's secrets by going into your project's Github repo, then clicking
"Settings" > "Secrets and Variables" > "Actions" and creating a "New repository secret" with the name "FLY_API_TOKEN".

To generate a `fly.toml` config file for deployment to Fly.io, you must first run the following from within the project source directory

```sh
fly launch --no-deploy
```

to create a new Fly app and register it with the service. Git commit your new `fly.toml` file.

To set up the Github Actions deployment workflow, copy the following into a `.github/workflows/fly_deploy.yml` file:


```admonish example collapsible=true

	# For more details, see: https://fly.io/docs/app-guides/continuous-deployment-with-github-actions/

	name: Deploy to Fly.io
	on:
	push:
		branches:
		- main
	jobs:
	deploy:
		name: Deploy app
		runs-on: ubuntu-latest
		steps:
		- uses: actions/checkout@v4
		- uses: superfly/flyctl-actions/setup-flyctl@master
		- name: Deploy to fly
			id: deployment
			run: |
			  flyctl deploy --remote-only | tail -n 1 >> $GITHUB_STEP_SUMMARY
			env:
			  FLY_API_TOKEN: ${{ secrets.FLY_API_TOKEN }}

```

On the next commit to your Github `main` branch, your project will automatically deploy to Fly.io.


## Deploy to Shuttle.rs



[Leptos Axum Starter Template for Shuttle.rs](https://github.com/Rust-WASI-WASM/shuttle-leptos-axum)

## Deploy to AWS Lambda

[Leptos Axum Starter Template for AWS Lambda](https://github.com/leptos-rs/start-aws)