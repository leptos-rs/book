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
particular JS framework like React may be hard to integrated. Libraries that manipulate DOM state in some way (for example, 
rich text editors) should also be used with care: both Leptos and the JS library will probably assume that they are 
the ultimate source of truth for the app’s state, so you should be careful to separate their responsibilities.

## Acccessing Web APIs with `web-sys`

If you just need to access some browser APIs without pulling in a separate JS library, you can do so using the 
[`web_sys`](https://docs.rs/web-sys/latest/web_sys/) crate. This provides bindings for all of the Web APIs provided by 
the browser, with 1:1 mappings from browser types and functions to Rust structs and methods.

In general, if you’re asking “how do I *do X* with Leptos?” where *do X* is accessing some Web API, looking up a vanilla
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
[`NodeRef`](https://docs.rs/leptos/latest/leptos/struct.NodeRef.html) type.

You may notice that [`NodeRef::get`](https://docs.rs/leptos/latest/leptos/struct.NodeRef.html#method.get) returns an `Option<leptos::HtmlElement<T>>`. This is *not* the same type as a [`web_sys::HtmlElement`](https://docs.rs/web-sys/latest/web_sys/struct.HtmlElement.html), although they 
are related. So what is this [`HtmlElement<T>`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html) type, and how do you use it?

### Overview

`web_sys::HtmlElement` is the Rust equivalent of the browser’s [`HTMLElement`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement) 
interface, which is implemented for all HTML elements. It provides access to a minimal set of functions and APIs that are guaranteed to be 
available for any HTML element. Each particular HTML element then has its own element class, which implements additional functionality. 
The goal of `leptos::HtmlElement<T>` is to bridge the gap between elements in your view and these more specific JavaScript types, so that you
can access the particular functionality of those elements.

This is implement by using the Rust `Deref` trait to allow you to dereference a `leptos::HtmlElement<T>` to the appropriately-typed JS object
for that particular element type `T`.

### Definition

Understanding this relationship involves understanding some related traits.

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

> See also: [The `wasm-bindgen` Guide: Inheritance in `web-sys`](https://rustwasm.github.io/wasm-bindgen/web-sys/inheritance.html).

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

> See the [`event_target_*` functions here](https://docs.rs/leptos/latest/leptos/fn.event_target.html?search=event_target), if you're curious.

### `leptos::HtmlElement`

The [`leptos::HtmlElement`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html) adds some extra convenience methods to make it easier to manipulate common attributes.
These methods were built for the [builder syntax](./view/builder.md), so it takes and returns `self`.
You can just do `_ = element.clone().<method>()` to ignore the element it returns - it'll still affect the original element, even though it doesn't look like it (see previous section on [Clones](#clones))!

Here are some of the common methods you may want to use, for example in event listeners or `use:` directives.
- [`id`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.id): *overwrites* the id on the element.
- [`classes`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.classes): *adds* the classes to the element.
    You can specify multiple classes with a space-separated string.
    You can also use [`class`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.class) to conditionally add a *single* class: do not add multiple with this method.
- [`attr`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.attr): sets a `key=value` attribute to the element.
- [`prop`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.prop): sets a *property* on the element: see the distinction between [properties and attributes here](./view/05_forms.md#why-do-you-need-propvalue).
- [`on`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.on): adds an event listener to the element.
    Specify the event type through one of [`leptos::ev::*`](https://docs.rs/leptos/latest/leptos/ev/index.html) (it's the ones in all lowercase).
- [`child`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html#method.child): adds an element as the last child of the element.

Take a look at the rest of the [`leptos::HtmlElement`](https://docs.rs/leptos/latest/leptos/struct.HtmlElement.html) methods too. If none of them fit your requirements, also take a look at [`leptos-use`](https://leptos-use.rs/). Otherwise, you’ll have to use the `web_sys` APIs.
