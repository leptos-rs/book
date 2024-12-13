# Hydration Bugs _(and how to avoid them)_

## A Thought Experiment

Let’s try an experiment to test your intuitions. Open up an app you’re server-rendering with `cargo-leptos`. (If you’ve just been using `trunk` so far to play with examples, go [clone a `cargo-leptos` template](./21_cargo_leptos.md) just for the sake of this exercise.)

Put a log somewhere in your root component. (I usually call mine `<App/>`, but anything will do.)

```rust
#[component]
pub fn App() -> impl IntoView {
	logging::log!("where do I run?");
	// ... whatever
}
```

And let’s fire it up

```bash
cargo leptos watch
```

Where do you expect `where do I run?` to log?

- In the command line where you’re running the server?
- In the browser console when you load the page?
- Neither?
- Both?

Try it out.

...

...

...

Okay, consider the spoiler alerted.

You’ll notice of course that it logs in both places, assuming everything goes according to plan. In fact on the server it logs twice—first during the initial server startup, when Leptos renders your app once to extract the route tree, then a second time when you make a request. Each time you reload the page, `where do I run?` should log once on the server and once on the client.

If you think about the description in the last couple sections, hopefully this makes sense. Your application runs once on the server, where it builds up a tree of HTML which is sent to the client. During this initial render, `where do I run?` logs on the server.

Once the WASM binary has loaded in the browser, your application runs a second time, walking over the same user interface tree and adding interactivity.

> Does that sound like a waste? It is, in a sense. But reducing that waste is a genuinely hard problem. It’s what some JS frameworks like Qwik are intended to solve, although it’s probably too early to tell whether it’s a net performance gain as opposed to other approaches.

## The Potential for Bugs

Okay, hopefully all of that made sense. But what does it have to do with the title of this chapter, which is “Hydration bugs (and how to avoid them)”?

Remember that the application needs to run on both the server and the client. This generates a few different sets of potential issues you need to know how to avoid.

### Mismatches between server and client code

One way to create a bug is by creating a mismatch between the HTML that’s sent down by the server and what’s rendered on the client. It’s actually fairly hard to do this unintentionally, I think (at least judging by the bug reports I get from people.) But imagine I do something like this

```rust
#[component]
pub fn App() -> impl IntoView {
    let data = if cfg!(target_arch = "wasm32") {
        vec![0, 1, 2]
    } else {
        vec![]
    };
    data.into_iter()
        .map(|value| view! { <span>{value}</span> })
        .collect_view()
}
```

In other words, if this is being compiled to WASM, it has three items; otherwise it’s empty.

When I load the page in the browser, I see nothing. If I open the console I see a panic:

```
ssr_modes.js:423 panicked at /.../tachys/src/html/element/mod.rs:352:14:
called `Option::unwrap()` on a `None` value
```

The WASM version of your app, running in the browser, is expecting to find an element (in fact, it’s expecting three elements!) But the HTML sent from the server has none.

#### Solution

It’s pretty rare that you do this intentionally, but it could happen from somehow running different logic on the server and in the browser. If you’re seeing warnings like this and you don’t think it’s your fault, it’s much more likely that it’s a bug with `<Suspense/>` or something. Feel free to go ahead and open an [issue](https://github.com/leptos-rs/leptos/issues) or [discussion](https://github.com/leptos-rs/leptos/discussions) on GitHub for help.

### Invalid/edge-case HTML, and mismatches between HTML and the DOM

Servers respond to requests with HTML. The browser then parses that HTML into a tree called the Document Object Model (DOM). During hydration, Leptos walks over the view tree of your application, hydrating an element, then moving into its children, hydrating the first child, then moving to its siblings, and so on. This assumes that the tree of HTML produced by the your application on the server maps directly onto the DOM tree into which the browser parses that HTML.

There are a few cases to be aware of in which the tree of HTML created by your `view` and the DOM tree might not correspond exactly: these can cause hydration errors.

#### Invalid HTML

Here’s a very simple application that causes a hydration error:
```rust
#[component]
pub fn App() -> impl IntoView {
    let count = RwSignal::new(0);

    view! {
        <p>
            <div class:blue=move || count.get() == 2>
                 "First"
            </div>
        </p>
    }
}
```

This will give an error message like 
```
A hydration error occurred while trying to hydrate an element defined at src/app.rs:6:14.

The framework expected a text node, but found this instead:  <p></p>

The hydration mismatch may have occurred slightly earlier, but this is the first time the framework found a node of an unexpected type.
```

(In most browser devtools, you can right-click on that `<p></p>` to show where it appears in the DOM, which is handy.)

If you look in the DOM inspector, you’ll see that it instead of a `<div>` inside a `<p>`, it shows:
```html
<p></p>
<div>First</div>
<p></p>
```
That’s because this is invalid HTML! A `<div>` cannot go inside a `<p>`. When the browser parses that `<div>`, it actually closes the preceding `<p>`, then opens the `<div>`; then, when it sees the (now-unmatched) closing `</p>`, it treats it as a new, empty `<p>`.

As a result, our DOM tree no longer matches the expected view tree, and a hydration error ensues.

Unfortunately, it is difficult to ensure the validity of HTML in the view at compile time using our current model, and without an effect on compile times across the board. For now, if you run into issues like this, consider running the HTML output through a validator. (In the case above, the W3C HTML Validator does in fact show an error!)

```admonish info
You may notice some bugs of this arise when migrating from 0.6 to 0.7. This is due to a change in how hydration works.

Leptos 0.1-0.6 used a method of hydration in which each HTML element was given a unique ID, which was then used to find it in the DOM by ID. Leptos 0.7 instead began walking over the DOM directly, hydrating each element as it came. This has much better performance characteristics (shorter, cleaner HTML output and faster hydration times) but is less resilient to the invalid or edge-case HTML examples above. Perhaps more importantly, this approach also fixes a number of *other* edge cases and bugs in hydration, making the framework more resilient on net.
```

#### `<table>` without `<tbody>`

There’s one additional edge case I’m aware of, in which *valid* HTML produces a DOM tree that differs from the view tree, and that’s `<table>`. When (most) browsers parse an HTML `<table>`, they insert a `<tbody>` into the DOM, whether you included one or not.

```rust
#[component]
pub fn App() -> impl IntoView {
    let count = RwSignal::new(0);

    view! {
        <table>
            <tr>
                <td class:blue=move || count.get() == 0>"First"</td>
            </tr>
        </table>
    }
}
```

Again, this generates a hydration error, because the browser has inserted an additional `<tbody>` into the DOM tree that was not in your view.

Here, the fix is simple: adding `<tbody>`:
```rust
#[component]
pub fn App() -> impl IntoView {
    let count = RwSignal::new(0);

    view! {
        <table>
            <tbody>
                <tr>
                    <td class:blue=move || count.get() == 0>"First"</td>
                </tr>
            </tbody>
        </table>
    }
}
```

(It would be worth exploring in the future whether we can lint for this particular quirk more easily than linting for valid HTML.)

#### General Advice

These kind of mismatches can be tricky. In general, my recommendation for debugging:
1. Right-click on the element in the message to see where the framework first *notices* the problem.
2. Compare the DOM at that point and above it, checking for mismatches with your view tree. Are there extra elements? Missing elements?


### Not all client code can run on the server

Imagine you happily import a dependency like `gloo-net` that you’ve been used to using to make requests in the browser, and use it in a `create_resource` in a server-rendered app.

You’ll probably instantly see the dreaded message

```
panicked at 'cannot call wasm-bindgen imported functions on non-wasm targets'
```

Uh-oh.

But of course this makes sense. We’ve just said that your app needs to run on the client and the server.

#### Solution

There are a few ways to avoid this:

1. Only use libraries that can run on both the server and the client. [`reqwest`](https://docs.rs/reqwest/latest/reqwest/), for example, works for making HTTP requests in both settings.
2. Use different libraries on the server and the client, and gate them using the `#[cfg]` macro. ([Click here for an example](https://github.com/leptos-rs/leptos/blob/main/examples/hackernews/src/api.rs).)
3. Wrap client-only code in `Effect::new`. Because effects only run on the client, this can be an effective way to access browser APIs that are not needed for initial rendering.

For example, say that I want to store something in the browser’s `localStorage` whenever a signal changes.

```rust
#[component]
pub fn App() -> impl IntoView {
    use gloo_storage::Storage;
	let storage = gloo_storage::LocalStorage::raw();
	logging::log!("{storage:?}");
}
```

This panics because I can’t access `LocalStorage` during server rendering.

But if I wrap it in an effect...

```rust
#[component]
pub fn App() -> impl IntoView {
    use gloo_storage::Storage;
    Effect::new(move |_| {
        let storage = gloo_storage::LocalStorage::raw();
		log!("{storage:?}");
    });
}
```

It’s fine! This will render appropriately on the server, ignoring the client-only code, and then access the storage and log a message on the browser.

### Not all server code can run on the client

WebAssembly running in the browser is a pretty limited environment. You don’t have access to a file-system or to many of the other things the standard library may be used to having. Not every crate can even be compiled to WASM, let alone run in a WASM environment.

In particular, you’ll sometimes see errors about the crate `mio` or missing things from `core`. This is generally a sign that you are trying to compile something to WASM that can’t be compiled to WASM. If you’re adding server-only dependencies, you’ll want to mark them `optional = true` in your `Cargo.toml` and then enable them in the `ssr` feature definition. (Check out one of the template `Cargo.toml` files to see more details.)

You can use `create_effect` to specify that something should only run on the client, and not in the server. Is there a way to specify that something should run only on the server, and not the client?

In fact, there is. The next chapter will cover the topic of server functions in some detail. (In the meantime, you can check out their docs [here](https://docs.rs/leptos/0.7.0-gamma3/leptos/attr.server.html).)
