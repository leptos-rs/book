# Deploying a Full-Stack SSR App

It's possible to deploy Leptos fullstack, SSR apps to any number of server or container hosting services. The most simple way to get a Leptos SSR app into production might be to use a VPS service and either run Leptos natively in a VM ([see here for more details](https://github.com/leptos-rs/start-axum?tab=readme-ov-file#executing-a-server-on-a-remote-machine-without-the-toolchain)). Alternatively, you could containerize your Leptos app and run it in [Podman](https://podman.io/) or [Docker](https://www.docker.com/) on any colocated or cloud server.

There are a multitude of different deployment setups and hosting services, and in general, Leptos itself is agnostic to the deployment setup you use. With this diversity of deployment targets in mind, on this page we will go over:

- [creating a `Containerfile` (or `Dockerfile`) for use with Leptos SSR apps](#creating-a-containerfile)
- Using a `Dockerfile` to [deploy to a cloud service](#cloud-deployments) - [for example, Fly.io](#deploy-to-flyio)
- Deploying Leptos to [serverless runtimes](#deploy-to-serverless-runtimes) - for example, [AWS Lambda](#aws-lambda) and [JS-hosted WASM runtimes like Deno & Cloudflare](#deno--cloudflare-workers)
- [Platforms that have not yet gained Leptos SSR support](#currently-unsupported-platforms)

_Note: Leptos does not endorse the use of any particular method of deployment or hosting service._

## Creating a Containerfile

The most popular way for people to deploy full-stack apps built with `cargo-leptos` is to use a cloud hosting service that supports deployment via a Podman or Docker build. Here’s a sample `Containerfile` / `Dockerfile`, which is based on the one we use to deploy the Leptos website.

### Debian

```dockerfile
# Get started with a build env with Rust nightly
FROM rustlang/rust:nightly-trixie as builder

# If you’re using stable, use this instead
# FROM rust:1.92.0-trixie as builder # See current official Rust tags here: https://hub.docker.com/_/rust

# Install cargo-binstall, which makes it easier to install other
# cargo extensions like cargo-leptos
RUN wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN tar -xvf cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN cp cargo-binstall /usr/local/cargo/bin

# Install required tools
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends clang

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

FROM debian:trixie-slim as runtime
WORKDIR /app
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

# -- NB: update binary name from "leptos_start" to match your app name in Cargo.toml --
# Copy the server binary to the /app directory
COPY --from=builder /app/target/release/leptos_start /app/

# /target/site contains our JS/WASM/CSS, etc.
COPY --from=builder /app/target/site /app/site

# Copy Cargo.toml if it’s needed at runtime
COPY --from=builder /app/Cargo.toml /app/

# Set any required env variables and
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"
EXPOSE 8080

# -- NB: update binary name from "leptos_start" to match your app name in Cargo.toml --
# Run the server
CMD ["/app/leptos_start"]
```

### Alpine

```dockerfile
# Get started with a build env with Rust nightly
FROM rustlang/rust:nightly-alpine as builder

RUN apk update && \
    apk add --no-cache bash curl npm libc-dev binaryen

RUN npm install -g sass

RUN curl --proto '=https' --tlsv1.3 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/latest/download/cargo-leptos-installer.sh | sh

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /work
COPY . .

RUN cargo leptos build --release -vv

FROM rustlang/rust:nightly-alpine as runner

WORKDIR /app

COPY --from=builder /work/target/release/leptos_start /app/
COPY --from=builder /work/target/site /app/site
COPY --from=builder /work/Cargo.toml /app/

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT=./site
EXPOSE 8080

CMD ["/app/leptos_start"]
```

> Read more: [`gnu` and `musl` build files for Leptos apps](https://github.com/leptos-rs/leptos/issues/1152#issuecomment-1634916088).

## Cloud Deployments

### Deploy to Fly.io

One option for deploying your Leptos SSR app is to use a service like [Fly.io](https://fly.io/), which takes a Dockerfile definition of your Leptos app and runs it in a quick-starting micro-VM; Fly also offers a variety of storage options and managed DBs to use with your projects. The following example will show how to deploy a simple Leptos starter app, just to get you up and going; [see here for more about working with storage options on Fly.io](https://fly.io/docs/database-storage-guides/) if and when required.

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

```admonish note
By default, Fly.io will auto-stop machines that don't have traffic coming to them after a certain period of time. Although Fly.io's lightweight VM's start up quickly, if you want to minimize the latency of your Leptos app and ensure it's always swift to respond, go into the generated `fly.toml` file and change the `min_machines_running` to 1 from the default of 0.

[See this page in the Fly.io docs for more details](https://fly.io/docs/apps/autostart-stop/).
```

If you prefer to use Github Actions to manage your deployments, you will need to create a new access token via the [Fly.io](https://fly.io/) web UI.

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

See [the example repo here](https://github.com/diversable/fly-io-leptos-ssr-test-deploy).

### Railway

Another provider for cloud deployments is [Railway](https://railway.app/).
Railway integrates with GitHub to automatically deploy your code.

There is an opinionated community template that gets you started quickly:

[![Deploy on Railway](https://railway.app/button.svg)](https://railway.app/template/pduaM5?referralCode=fZ-SY1)

The template has renovate setup to keep dependencies up to date and supports GitHub Actions to test your code before a deploy happens.

Railway has a free tier that does not require a credit card, and with how little resources Leptos needs that free tier should last a long time.

See [the example repo here](https://github.com/marvin-bitterlich/leptos-railway).

## Deploy to Serverless Runtimes

Leptos supports deploying to FaaS (Function as a Service) or 'serverless' runtimes such as AWS Lambda as well as [WinterCG](https://wintercg.org/)-compatible JS runtimes such as [Deno](https://deno.com/deploy) and Cloudflare. Just be aware that serverless environments do place some restrictions on the functionality available to your SSR app when compared with VM or container type deployments (see notes, below).

### AWS Lambda

With a little help from the [Cargo Lambda](https://www.cargo-lambda.info/) tool, Leptos SSR apps can be deployed to AWS Lambda. A starter template repo using Axum as the server is available at [leptos-rs/start-aws](https://github.com/leptos-rs/start-aws); the instructions there can be adapted for you to use a Leptos+Actix-web server as well. The starter repo includes a Github Actions script for CI/CD, as well as instructions for setting up your Lambda functions and getting the necessary credentials for cloud deployment.

However, please keep in mind that some native server functionality does not work with FaaS services like Lambda because the environment is not necessarily consistent from one request to the next. In particular, the ['start-aws' docs](https://github.com/leptos-rs/start-aws#state) state that "since AWS Lambda is a serverless platform, you'll need to be more careful about how you manage long-lived state. Writing to disk or using a state extractor will not work reliably across requests. Instead, you'll need a database or other microservices that you can query from the Lambda function."

The other factor to bear in mind is the 'cold-start' time for functions as a service - depending on your use case and the FaaS platform you use, this may or may not meet your latency requirements; you may need to keep one function running at all times to optimize the speed of your requests.

### Deno & Cloudflare Workers

Currently, Leptos-Axum supports running in Javascript-hosted WebAssembly runtimes such as Deno, Cloudflare Workers, etc. This option requires some changes to the setup of your source code (for example, in `Cargo.toml` you must define your app using `crate-type = ["cdylib"]` and the "wasm" feature must be enabled for `leptos_axum`). [The Leptos HackerNews JS-fetch example](https://github.com/leptos-rs/leptos/tree/leptos_0.6/examples/hackernews_js_fetch) demonstrates the required modifications and shows how to run an app in the Deno runtime. Additionally, the [`leptos_axum` crate docs](https://docs.rs/leptos_axum/latest/leptos_axum/#js-fetch-integration) are a helpful reference when setting up your own `Cargo.toml` file for JS-hosted WASM runtimes.

While the initial setup for JS-hosted WASM runtimes is not onerous, the more important restriction to keep in mind is that since your app will be compiled to WebAssembly (`wasm32-unknown-unknown`) on the server as well as the client, you must ensure that the crates you use in your app are all WASM-compatible; this may or may not be a deal-breaker depending on your app's requirements, as not all crates in the Rust ecosystem have WASM support.

If you're willing to live with the limitations of WASM server-side, the best place to get started right now is by checking out the [example of running Leptos with Deno](https://github.com/leptos-rs/leptos/tree/leptos_0.6/examples/hackernews_js_fetch) in the official Leptos Github repo.

## Platforms Working on Leptos Support

### Deploy to Spin Serverless WASI (with Leptos SSR)

WebAssembly on the server has been gaining steam lately, and the developers of the open source serverless WebAssembly framework Spin are working on natively supporting Leptos. While the Leptos-Spin SSR integration is still in its early stages, there is a working example you may wish to try out.

The full set of instructions to get Leptos SSR & Spin working together are available as [a post on the Fermyon blog](https://www.fermyon.com/blog/leptos-spin-get-started), or if you want to skip the article and just start playing around with a working starter repo, [see here](https://github.com/diversable/leptos-spin-ssr-test).

### Deploy to Shuttle.rs

Several Leptos users have asked about the possibility of using the Rust-friendly [Shuttle.rs](https://www.shuttle.rs/) service to deploy Leptos apps. Unfortunately, Leptos is not officially supported by the Shuttle.rs service at the moment.

However, the folks at Shuttle.rs are committed to getting Leptos support in the future; if you would like to keep up-to-date on the status of that work, keep an eye on [this Github issue](https://github.com/shuttle-hq/shuttle/issues/1002#issuecomment-1853661643).

Additionally, some effort has been made to get Shuttle working with Leptos, but to date, deploys to the Shuttle cloud are still not working as expected. That work is available here, if you would like to investigate for yourself or contribute fixes: [Leptos Axum Starter Template for Shuttle.rs](https://github.com/Rust-WASI-WASM/shuttle-leptos-axum).
