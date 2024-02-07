# `web_sys` and `HtmlElement`

The [`web_sys`](https://docs.rs/web-sys/latest/web_sys/) crate provides bindings to JavaScript web APIs that you can call in your Rust code.
If possible, you probably want to use something provided by Leptos, or the fantastic utilities crate [`leptos-use`](https://leptos-use.rs/).
However, if neither provides the API you are looking for, you will have to fall back to using `web_sys`.
Here are a few things to keep in mind while using it.

*Supplementary reading: [The `wasm-bindgen` Guide: `web-sys`](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/index.html)*

## Enabling features

`web_sys` is heavily feature-gated to keep compile times low.
If you would like to use one of its many APIs, you may need to enable a feature to use it.

The features required to use an item is always listed in it's documentation.
For example, to use [`Element::get_bounding_rect_client`](https://docs.rs/web-sys/latest/web_sys/struct.Element.html#method.get_bounding_client_rect), you need to enable the `DomRect` and `Element` features.

Leptos already enables [a whole bunch](https://github.com/leptos-rs/leptos/blob/main/leptos_dom/Cargo.toml#L41) of features - if the required feature is already enabled here, you won't have to enable it in your own app.
Otherwise, add it to your `Cargo.toml` and you're good to go!

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

## `web_sys::HtmlElement`

If you've used the [`NodeRef`](https://docs.rs/leptos/latest/leptos/struct.NodeRef.html) type, you may notice that [`get`](https://docs.rs/leptos/latest/leptos/struct.NodeRef.html#method.get)ting it returns an `Option<leptos::HtmlElement<T>>`.
What is this [`HtmlElement<T>`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html) type, and how do you use it?

While related, `leptos::HtmlElement<T>` and [`web_sys::HtmlElement`](https://docs.rs/web-sys/latest/web_sys/struct.HtmlElement.html) are different types.

### Definition

There's a bunch of related structs and traits, so I'll gather all the relevant ones here.

The following simply defines what types are allowed inside the `T` of `leptos::HtmlElement<T>` and how it links to `web_sys`.

```rust
pub struct HtmlElement<El> where El: ElementDescriptor { /* ... */ }

pub trait ElementDescriptor: ElementDescriptorBounds { /* ... */ }

pub trait ElementDescriptorBounds: Debug {}
impl<El> ElementDescriptorBounds for El where El: Debug {}

// this is implemented for every single element in `leptos::{html, svg, math}::*`
impl ElementDescriptor for leptos::html::Div { /* ... */ }

// same with this, derefs to the corresponding `web_sys::Html*Element`
impl Deref for leptos::html::Div {
    type Target = web_sys::HtmlDivElement;
    // ...
}
```

The following is from `web_sys`:
```rust
impl Deref for web_sys::HtmlDivElement {
    type Target = web_sys::HtmlElement;
    // ...
}

impl Deref for web_sys::HtmlElement {
    type Target = web_sys::Element;
    // ...
}

impl Deref for web_sys::Element {
    type Target = web_sys::Node;
    // ...
}

impl Deref for web_sys::Node {
    type Target = web_sys::EventTarget;
    // ...
}
```

`web_sys` uses long deref chains to emulate the inheritance used in JavaScript.
If you can't find the method you're looking for on one type, take a look further down the deref chain.
The `leptos::html::*` types all deref into `web_sys::Html*Element` or `web_sys::HtmlElement`.
By calling `element.method()`, Rust will automatically add more derefs as needed to call the correct method!

However, some methods have the same name, such as [`leptos::HtmlElement::style`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.style) and [`web_sys::HtmlElement::style`](https://docs.rs/web-sys/latest/web_sys/struct.HtmlElement.html#method.style).
In these cases, Rust will pick the one that requires the least amount of derefs, which is `leptos::HtmlElement::style` if you're getting an element straight from a `NodeRef`.
If you wish to use the `web_sys` method instead, you can manually deref with `(*element).style()`.

If you want to have even more control over which type you are calling a method from, `AsRef<T>` is implemented for all types that are part of the deref chain, so you can explicitly state which type you want.

*See also: [The `wasm-bindgen` Guide: Inheritance in `web-sys`](https://rustwasm.github.io/wasm-bindgen/web-sys/inheritance.html).*

### Clones

The `web_sys::HtmlElement` (and by extension the `leptos::HtmlElement` too) actually only store references to the HTML element it affects.
Therefore, calling `.clone()` doesn't actually make a new HTML element, it simply gets another reference to the same one.
Calling methods that change the element from any of its clones will affect the original element.

Unfortunately, `web_sys::HtmlElement` does not implement `Copy`, so you may need to add a bunch of clones especially when using it in closures.
Don't worry though, these clones are cheap!

### Casting

You can get less specific types through `Deref` or `AsRef`, so use those when possible.
However, if you need to cast to a more specific type (e.g. from an `EventTarget` to a `HtmlInputElement`), you will need to use the methods provided by `wasm_bindgen::JsCast` (re-exported through `web_sys::wasm_bindgen::JsCast`).
You'll probably only need the [`dyn_ref`](https://docs.rs/wasm-bindgen/0.2.90/wasm_bindgen/trait.JsCast.html#method.dyn_ref) method.

```rust
use web_sys::wasm_bindgen::JsCast;

let on_click = |ev: MouseEvent| {
    let target: HtmlInputElement = ev.current_target().unwrap().dyn_ref().unwrap();
    // or, just use the existing `leptos::event_target_*` functions
}
```

*See the [`event_target_*` functions here](https://docs.rs/leptos/latest/leptos/fn.event_target.html?search=event_target), if you're curious.*

### `leptos::HtmlElement`

The [`leptos::HtmlElement`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html) adds some extra convenience methods to make it easier to manipulate common attributes.
These methods were built for the [builder syntax](./builder.md), so it takes and returns `self`.
You can just do `_ = element.clone().<method>()` to ignore the element it returns - it'll still affect the original element, even though it doesn't look like it (see previous section on [Clones](#clones))!

Here are some of the common methods you may want to use, for example in event listeners or `use:` directives.
- [`id`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.id): *overwrites* the id on the element.
- [`classes`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.classes): *adds* the classes to the element.
    You can specify multiple classes with a space-separated string.
    You can also use [`class`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.class) to conditionally add a *single* class: do not add multiple with this method.
- [`attr`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.attr): sets a `key=value` attribute to the element.
- [`prop`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.prop): sets a *property* on the element: see the distinction between [properties and attributes here](./05_forms.md#why-do-you-need-propvalue).
- [`on`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.on): adds an event listener to the element.
    Specify the event type through one of [`leptos::ev::*`](https://docs.rs/leptos/latest/leptos/ev/index.html) (it's the ones in all lowercase).
- [`child`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.child): adds an element as the last child of the element.

Take a look at the rest of the [`leptos::HtmlElement`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html) methods too - if none of them fit your requirements, also take a look at [`leptos-use`](https://leptos-use.rs/).
Otherwise, you'll have to use the `web_sys` APIs.
