# Part 1: Building User Interfaces

In the first part of the book, we're going to look at building user interfaces on the client-side using Leptos. Under the hood, Leptos and Trunk are bundling up a snippet of Javascript which will load up the Leptos UI, which has been compiled to WebAssembly to drive the interactivity in your CSR (client-side rendered) website.

Part 1 will introduce you to the basic tools you need to build a reactive user interface powered by Leptos and Rust. By the end of Part 1, you should be able to
build a snappy synchronous website that's rendered in the browser and which you can deploy on any static-site hosting service, like Github Pages or Vercel.

```admonish info
To get the most out of this book, we encourage you to code along with the examples provided.
In the [Getting Started](https://book.leptos.dev/getting_started/) and [Leptos DX](https://book.leptos.dev/getting_started/leptos_dx.html) chapters, we showed you how to set up a basic project with Leptos and Trunk, including WASM error handling in the browser.
That basic setup is enough to get you started developing with Leptos.

If you'd prefer to get started using a more full-featured template which demonstrates how to set up a few of the basics you'd see in a real Leptos project, such as routing, (covered later in the book), injecting `<Title>` and `<Meta>` tags into the page head, and a few other niceties, then feel free to utilize [the leptos-rs `start-trunk`](https://github.com/leptos-rs/start-trunk) template repo to get up and running.

The `start-trunk` template requires that you have `Trunk` and `cargo-generate` installed, which you can get by running `cargo install trunk` and `cargo install cargo-generate`.

To use the template to set up your project, just run

`cargo generate --git https://github.com/leptos-rs/start-trunk`

then run

`trunk serve --port 3000 --open`

in the newly created app's directory to start developing your app.
The Trunk server will reload your app on file changes, making development relatively seamless.

```
