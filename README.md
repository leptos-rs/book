# Leptos Book

- [Leptos Book](#leptos-book)
  - [Introduction](#introduction)
  - [Building the Book](#building-the-book)
  - [Optional: VSCode Dev Container](#vscode-dev-container)

## Introduction

This project contains the core of a new introductory guide to Leptos. Pull requests for any typos, clarification, or improvements are always welcome.

You can find the live version of this book on the [Leptos Website](https://book.leptos.dev/).

## Building the Book

### VSCode Dev Container

The easiest way to build the book is to use the included [VSCode Dev Container](https://code.visualstudio.com/docs/devcontainers/containers). This will automatically install all dependencies, build the book, and serve it at [`http://localhost:3000`](http://localhost:3000) with live reloading for your convenience.

Simply open the repository in VSCode and click the "Reopen in Container" button in the bottom right corner of the window when VSCode loads. You will need to have [Docker](https://www.docker.com/) installed for this to work and you will also need to install the official [Dev Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) extension.

### Manual Build

If you don't want to use the Dev Container, you can build the book manually. You must have Rust installed to do this.

#### Installing Dependencies

It is built using [`mdbook`](https://crates.io/crates/mdbook). You can view a local copy by installing `mdbook` with Cargo.

```sh
cargo install mdbook --version 0.4.*
```

This book also uses an mdbook preprocessor called [`mdbook-admonish`](https://crates.io/crates/mdbook-admonish) to style blocks of text like notes, warnings, etc.

```sh
cargo install mdbook-admonish --version 1.*
```


You can now build the book with live reloading by running the following command in the root of the repository.

```sh
mdbook serve
```

It should now be available at [`http://localhost:3000`](http://localhost:3000).
