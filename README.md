# Leptos Book

- [Leptos Book](#leptos-book)
  - [Introduction](#introduction)
  - [Building the Book](#building-the-book)
  - [Optional: VSCode Dev Container](#vscode-dev-container)

## Introduction

This project contains the core of a new introductory guide to Leptos. Pull requests for any typos, clarification, or improvements are always welcome.

You can find the live version of this book on the [Leptos Website](https://book.leptos.dev/).

## Building the Book

It is built using [`mdbook`](https://crates.io/crates/mdbook). You can view a local copy by installing `mdbook` with Cargo.

```sh
cargo install mdbook --version 0.4.*
```

This book also uses an mdbook preprocessor called [`mdbook-admonish`](https://crates.io/crates/mdbook-admonish) to style blocks of text like notes, warnings, etc.

```sh
cargo install mdbook-admonish --version 1.*
```


and then run the book with

```sh
mdbook serve
```

It should now be available at [`http://localhost:3000`](http://localhost:3000).

## Optional: VSCode Dev Container

You can optionally build and run it in the example [VSCode Dev Container](https://code.visualstudio.com/docs/devcontainers/containers), which will automatically install all dependencies, build the book, and serve it at [`http://localhost:3000`](http://localhost:3000) with live reloading.

Install Docker and the official [Dev Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) extension, then open the project in VSCode and "Reopen in dev container" when prompted.

For more info, see: https://code.visualstudio.com/remote/advancedcontainers/use-docker-kubernetes

To run Docker commands inside the dev container, see: https://code.visualstudio.com/remote/advancedcontainers/use-docker-kubernetes
