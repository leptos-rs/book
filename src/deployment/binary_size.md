# Optimizing WASM Binary Size

WebAssembly binaries are significantly larger than the JavaScript bundles you’d expect for the equivalent application. Because the WASM format is designed for streaming compilation, WASM files are much faster to compile per kilobyte than JavaScript files. (For a deeper look, you can [read this great article from the Mozilla team](https://hacks.mozilla.org/2018/01/making-webassembly-even-faster-firefoxs-new-streaming-and-tiering-compiler/) on streaming WASM compilation.) Still, it’s important to ship the smallest WASM binary to users that you can, as it will reduce their network usage and make your app interactive as quickly as possible.

So what are some practical steps?

## Things to Do

1. Make sure you’re looking at a release build. (Debug builds are much, much larger.)
2. Add a release profile for WASM that optimizes for size, not speed.

For a `cargo-leptos` project, for example, you can add this to your `Cargo.toml`:

```toml
[profile.wasm-release]
inherits = "release"
opt-level = 'z'
lto = true
codegen-units = 1

# ....

[package.metadata.leptos]
# ....
lib-profile-release = "wasm-release"
```

This will hyper-optimize the WASM for your release build for size, while keeping your server build optimized for speed. (For a pure client-rendered app without server considerations, just use the `[profile.wasm-release]` block as your `[profile.release]`.)

3. Always serve compressed WASM in production. WASM tends to compress very well, typically shrinking to less than 50% its uncompressed size, and it’s trivial to enable compression for static files being served from Actix or Axum.

4. If you’re using nightly Rust, you can rebuild the standard library with this same profile rather than the prebuilt standard library that’s distributed with the `wasm32-unknown-unknown` target.

To do this, create a file in your project at `.cargo/config.toml`

```toml
[unstable]
build-std = ["std", "panic_abort", "core", "alloc"]
build-std-features = ["panic_immediate_abort"]
```

Note that if you're using this with SSR too, the same Cargo profile will be applied. You'll need to explicitly specify your target:
```toml
[build]
target = "x86_64-unknown-linux-gnu" # or whatever
```

Also note that in some cases, the cfg feature `has_std` will not be set, which may cause build errors with some dependencies which check for `has_std`. You may fix any build errors due to this by adding:
```toml
[build]
rustflags = ["--cfg=has_std"]
```

And you'll need to add `panic = "abort"` to `[profile.release]` in `Cargo.toml`. Note that this applies the same `build-std` and panic settings to your server binary, which may not be desirable. Some further exploration is probably needed here.

5. One of the sources of binary size in WASM binaries can be `serde` serialization/deserialization code. Leptos uses `serde` by default to serialize and deserialize resources created with `Resource::new()`. `leptos_server` includes additional features to activate alternative encodings by adding additional `new_` methods. For example, activating the `miniserde` feature on the `leptos_server` crate adds a `Resource::new_miniserde()` method, and the `serde-lite` feature adds `new_serde_lite`. `miniserde` and `serde-lite` only implement subsets of `serde`’s functionality, but typically optimize for binary size over speed.

## Things to Avoid

There are certain crates that tend to inflate binary sizes. For example, the `regex` crate with its default features adds about 500kb to a WASM binary (largely because it has to pull in Unicode table data!). In a size-conscious setting, you might consider avoiding regexes in general, or even dropping down and calling browser APIs to use the built-in regex engine instead. (This is what `leptos_router` does on the few occasions it needs a regular expression.)

In general, Rust’s commitment to runtime performance is sometimes at odds with a commitment to a small binary. For example, Rust monomorphizes generic functions, meaning it creates a distinct copy of the function for each generic type it’s called with. This is significantly faster than dynamic dispatch, but increases binary size. Leptos tries to balance runtime performance with binary size considerations pretty carefully; but you might find that writing code that uses many generics tends to increase binary size. For example, if you have a generic component with a lot of code in its body and call it with four different types, remember that the compiler could include four copies of that same code. Refactoring to use a concrete inner function or helper can often maintain performance and ergonomics while reducing binary size.

## Code Splitting

`cargo-leptos` and the Leptos framework and router have support for WASM binary splitting. (Note that this support was released during the summer of 2025; depending on when you’re reading this, we may still be ironing out bugs.)

This can be used through the combination of three tools: `cargo leptos (serve|watch|build) --split`, the [`#[lazy]`](https://docs.rs/leptos/latest/leptos/attr.lazy.html) macro, and the [`#[lazy_route]`](https://docs.rs/leptos_router/latest/leptos_router/attr.lazy_route.html) macro (paired with the [`LazyRoute`](https://docs.rs/leptos_router/latest/leptos_router/trait.LazyRoute.html) trait).

### `#[lazy]`

The `#[lazy]` macro indicates that a function can be lazy-loaded from a separate WebAssembly (WASM) binary. It can be used to annotate a synchronous or async function; in either case, it will produce an async function. The first time you call the lazy-loaded function, that separate chunk of code will be loaded from the server and called. Subsequently, it will be called without an additional loading step.

```rust
#[lazy]
fn lazy_synchronous_function() -> String {
    "Hello, lazy world!".to_string()
}

#[lazy]
async fn lazy_async_function() -> String {
    /* do something that requires async work */
    "Hello, lazy async world!".to_string()
}

async fn use_lazy_functions() {
    // synchronous function has been converted to async
    let value1 = lazy_synchronous_function().await;

    // async function is still async
    let value1 = lazy_async_function().await;
}
```

This can be useful for one-off lazy functions. But lazy-loading is most powerful when it’s paired with the router.

### `#[lazy_route]`

Lazy routes allow you to split out the code for a route’s view, and to lazily load it concurrently with data for that route while navigating. Through the use of nested routing, multiple lazy-loaded routes can be nested: each will load its own data and its own lazy view concurrently.

Splitting the data loading from the (lazy-loaded) view allows you to prevent a “waterfall,” in which you wait for the lazy view to load, then begin loading data.

```rust
use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};

// the route definition
#[derive(Debug)]
struct BlogListingRoute {
    titles: Resource<Vec<String>>
}

#[lazy_route]
impl LazyRoute for BlogListingRoute {
    fn data() -> Self {
        Self {
            titles: Resource::new(|| (), |_| async {
                vec![/* todo: load blog posts */]
            })
        }
    }

    // this function will be lazy-loaded, concurrently with data()
    fn view(this: Self) -> AnyView {
        let BlogListingRoute { titles } = this;

        // ... now you can use the `posts` resource with Suspense, etc.,
        // and return AnyView by calling .into_any() on a view
    }
}
```

### Examples and More Information

You can find more in-depth discussion in [this YouTube video](https://www.youtube.com/watch?v=w5fhcoxQnII), and a full [`lazy_routes`](https://github.com/leptos-rs/leptos/blob/main/examples/lazy_routes/src/app.rs) example in the repo.
