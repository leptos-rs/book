# Integrating with JavaScript: `wasm-bindgen`, `web_sys` and `HtmlElement`

Leptos provides a variety of tools to allow you to build declarative web applications without leaving the world
of the framework. Things like the reactive system, `component` and `view` macros, and router allow you to build
user interfaces without directly interacting with the Web APIs provided by the browser. And they let you do it
all directly in Rust, which is great—assuming you like Rust. (And if you’ve gotten this far in the book, we assume
you like Rust.)

Ecosystem crates like the fantastic set of utilities provided by [`leptos-use`](https://leptos-use.rs/) can take you
even further, by providing Leptos-specific reactive wrappers around many Web APIs.

Nevertheless, in many cases you will need to access JavaScript libraries or Web APIs directly. This chapter can help.

## Using JS Libraries with `wasm-bindgen`

Your Rust code can be compiled to a WebAssembly (WASM) module and loaded to run in the browser. However, WASM does not
have direct access to browser APIs. Instead, the Rust/WASM ecosystem depends on generating bindings from your Rust code
to the JavaScript browser environment that hosts it.

The [`wasm-bindgen`](https://rustwasm.github.io/docs/wasm-bindgen/) crate is at the center of that ecosystem. It provides
both an interface for marking parts of Rust code with annotations telling it how to call JS, and a CLI tool for generating
the necessary JS glue code. You’ve been using this without knowing it all along: both `trunk` and `cargo-leptos` rely on
`wasm-bindgen` under the hood.

If there is a JavaScript library that you want to call from Rust, you should refer to the `wasm-bindgen` docs on
[importing functions from JS](https://rustwasm.github.io/docs/wasm-bindgen/examples/import-js.html). It is relatively
easy to import individual functions, classes, or values from JavaScript to use in your Rust app.

It is not always easy to integrate JS libraries into your app directly. In particular, any library that depends on a
particular JS framework like React may be hard to integrate. Libraries that manipulate DOM state in some way (for example,
rich text editors) should also be used with care: both Leptos and the JS library will probably assume that they are
the ultimate source of truth for the app’s state, so you should be careful to separate their responsibilities.

## Accessing Web APIs with `web-sys`

If you just need to access some browser APIs without pulling in a separate JS library, you can do so using the
[`web_sys`](https://docs.rs/web-sys/latest/web_sys/) crate. This provides bindings for all of the Web APIs provided by
the browser, with 1:1 mappings from browser types and functions to Rust structs and methods.

In general, if you’re asking “how do I _do X_ with Leptos?” where _do X_ is accessing some Web API, looking up a vanilla
JavaScript solution and translating it to Rust using the [`web-sys` docs](https://docs.rs/web-sys/latest/web_sys/) is a
good approach.

> After this section, you might find
> [the `wasm-bindgen` guide chapter on `web-sys`](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/index.html)
> useful for additional reading.

### Enabling features

`web_sys` is heavily feature-gated to keep compile times low. If you would like to use one of its many APIs, you may
need to enable a feature to use it.

The features required to use an item are always listed in its documentation.
For example, to use [`Element::get_bounding_rect_client`](https://docs.rs/web-sys/latest/web_sys/struct.Element.html#method.get_bounding_client_rect), you need to enable the `DomRect` and `Element` features.

Leptos already enables [a whole bunch](https://github.com/leptos-rs/leptos/blob/main/leptos_dom/Cargo.toml#L41) of features - if the required feature is already enabled here, you won't have to enable it in your own app.
Otherwise, add it to your `Cargo.toml` and you’re good to go!

```toml
[dependencies.web-sys]
version = "0.3"
features = ["DomRect"]
```

However, as the JavaScript standard evolves and APIs are being written, you may want to use browser features that are technically not fully stable yet, such as [WebGPU](https://docs.rs/web-sys/latest/web_sys/struct.Gpu.html).
`web_sys` will follow the (potentially frequently changing) standard, which means that no stability guarantees are made.

In order to use this, you need to add `RUSTFLAGS=--cfg=web_sys_unstable_apis` as an environment variable.
This can either be done by adding it to every command, or add it to `.cargo/config.toml` in your repository.

As part of a command:

```sh
RUSTFLAGS=--cfg=web_sys_unstable_apis cargo # ...
```

In `.cargo/config.toml`:

```toml
[env]
RUSTFLAGS = "--cfg=web_sys_unstable_apis"
```

## Accessing raw `HtmlElement`s from your `view`

The declarative style of the framework means that you don’t need to directly manipulate DOM nodes to build up your user interface.
However, in some cases you want direct access to the underlying DOM element that represents part of your view. The section of the book
on [“uncontrolled inputs”](/view/05_forms.html?highlight=NodeRef#uncontrolled-inputs) showed how to do this using the
[`NodeRef`](https://docs.rs/leptos/0.7.0-gamma3/leptos/tachys/reactive_graph/node_ref/struct.NodeRef.html) type.

`NodeRef::get` returns a correctly-typed
`web-sys` element that can be directly manipulated.

For example, consider the following:

```rust
#[component]
pub fn App() -> impl IntoView {
    let node_ref = NodeRef::<Input>::new();

    Effect::new(move |_| {
        if let Some(node) = node_ref.get() {
            leptos::logging::log!("value = {}", node.value());
        }
    });

    view! {
        <input node_ref=node_ref/>
    }
}
```

Inside the effect here, `node` is simply a `web_sys::HtmlInputElement`. This allows us to call any appropriate methods.

(Note that `.get()` returns an `Option` here, because the `NodeRef` is empty until it is filled when the DOM elements are actually created. Effects run a tick after the component runs, so in most cases the `<input>` will already have been created by the time the effect runs.)
