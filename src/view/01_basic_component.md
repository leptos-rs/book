# A Basic Component

That “Hello, world!” was a _very_ simple example. Let’s move on to something a
little more like an ordinary app.

First, let’s edit the `main` function so that, instead of rendering the whole
app, it just renders an `<App/>` component. Components are the basic unit of
composition and design in most web frameworks, and Leptos is no exception.
Conceptually, they are similar to HTML elements: they represent a section of the
DOM, with self-contained, defined behavior. Unlike HTML elements, they are in
`PascalCase`, so most Leptos applications will start with something like an
`<App/>` component.

```rust
use leptos::mount::mount_to_body;

fn main() {
    mount_to_body(App);
}
```

Now let’s define our `App` component itself. Because it’s relatively simple,
I’ll give you the whole thing up front, then walk through it line by line.

```rust
use leptos::prelude::*;

#[component]
fn App() -> impl IntoView {
    let (count, set_count) = signal(0);

    view! {
        <button
            on:click=move |_| set_count.set(3)
        >
            "Click me: "
            {count}
        </button>
        <p>
            "Double count: "
            {move || count.get() * 2}
        </p>
    }
}
```

## Importing the Prelude

```rust
use leptos::prelude::*;
```

Leptos provides a prelude which includes commonly-used traits and functions.
If you'd prefer to use individual imports, feel free to do that; the compiler
will provide helpful recommendations for each import.

## The Component Signature

```rust
#[component]
```

Like all component definitions, this begins with the [`#[component]`](https://docs.rs/leptos/latest/leptos/attr.component.html) macro. `#[component]` annotates a function so it can be
used as a component in your Leptos application. We’ll see some of the other features of
this macro in a couple chapters.

```rust
fn App() -> impl IntoView
```

Every component is a function with the following characteristics

1. It takes zero or more arguments of any type.
2. It returns `impl IntoView`, which is an opaque type that includes
   anything you could return from a Leptos `view`.

> Component function arguments are gathered together into a single props struct
> which is built by the `view` macro as needed.

## The Component Body

The body of the component function is a set-up function that runs once, not a
render function that reruns multiple times. You’ll typically use it to create a
few reactive variables, define any side effects that run in response to those values
changing, and describe the user interface.

```rust
let (count, set_count) = signal(0);
```

[`signal`](https://docs.rs/leptos/latest/leptos/reactive/signal/fn.signal.html)
creates a signal, the basic unit of reactive change and state management in Leptos.
This returns a `(getter, setter)` tuple. To access the current value, you’ll
use `count.get()` (or, on `nightly` Rust, the shorthand `count()`). To set the
current value, you’ll call `set_count.set(...)` (or, on nightly, `set_count(...)`).

> `.get()` clones the value and `.set()` overwrites it. In many cases, it’s more efficient to use `.with()` or `.update()`; check out the docs for [`ReadSignal`](https://docs.rs/leptos/latest/leptos/reactive/signal/struct.ReadSignal.html) and [`WriteSignal`](https://docs.rs/leptos/latest/leptos/reactive/signal/struct.WriteSignal.html) if you’d like to learn more about those trade-offs at this point.

## The View

Leptos defines user interfaces using a JSX-like format via the [`view`](https://docs.rs/leptos/latest/leptos/macro.view.html) macro.

```rust
view! {
    <button
        // define an event listener with on:
        on:click=move |_| set_count.set(3)
    >
        // text nodes are wrapped in quotation marks
        "Click me: "

        // blocks include Rust code
        // in this case, it renders the value of the signal
        {count}
    </button>
    <p>
        "Double count: "
        {move || count.get() * 2}
    </p>
}
```

This should mostly be easy to understand: it mostly looks like HTML, with a special
`on:click` syntax to define a `click` event listener and a few text nodes that look like
Rust strings. All HTML elements are supported, including both built-in elements (like `<p>`)
and custom elements/web components (like `<my-custom-element>`).

```admonish info
**Unquoted text**: The `view` macro does have some support for unquoted text nodes, which are the
norm in HTML or JSX (i.e., `<p>Hello!</p>` rather than `<p>"Hello!"</p>`). Due to limitations of
Rust proc macros, using unquoted text can occasionally cause spacing issues around punctuation, and
does not support all Unicode strings. You can use unquoted text if it’s your preference; note that
if you encounter any issues with it, they can always be resolved by quoting the text node as an ordinary
Rust string.
```

Then there are two values in braces: one, `{count}`, seems pretty easy
to understand (it's just the value of our signal), and then...

```rust
{move || count.get() * 2}
```

whatever that is.

People sometimes joke that they use more closures in their first Leptos application
than they’ve ever used in their lives. And fair enough.

Passing a function into the view tells the framework: “Hey, this is something
that might change.”

When we click the button and call `set_count`, the `count` signal is updated. This
`move || count.get() * 2` closure, whose value depends on the value of `count`, reruns,
and the framework makes a targeted update to that specific text node, touching
nothing else in your application. This is what allows for extremely efficient updates
to the DOM.

Remember—and this is _very important_—only signals and functions are treated as reactive
values in the view.

This means that `{count}` and `{count.get()}` do very different things in your view.
`{count}` passes in a signal, telling the framework to update the view every time `count` changes.
`{count.get()}` accesses the value of `count` once, and passes an `i32` into the view,
rendering it once, unreactively.

In the same way, `{move || count.get() * 2}` and `{count.get() * 2}` behave differently.
The first one is a function, so it's rendered reactively. The second is a value, so it's
just rendered once, and won't update when `count` changes.

You can see the difference in the CodeSandbox below!

Let’s make one final change. `set_count.set(3)` is a pretty useless thing for a click handler to do. Let’s replace “set this value to 3” with “increment this value by 1”:

```rust
move |_| {
    *set_count.write() += 1;
}
```

You can see here that while `set_count` just sets the value, `set_count.write()` gives us a mutable reference and mutates the value in place. Either one will trigger a reactive update in our UI.

> Throughout this tutorial, we’ll use CodeSandbox to show interactive examples.
> Hover over any of the variables to show Rust-Analyzer details
> and docs for what’s going on. Feel free to fork the examples to play with them yourself!

```admonish sandbox title="Live example" collapsible=true

[Click to open CodeSandbox.](https://codesandbox.io/p/devbox/1-basic-component-0-7-qvgdxs?file=%2Fsrc%2Fmain.rs%3A1%2C1-59%2C2&workspaceId=478437f3-1f86-4b1e-b665-5c27a31451fb)

<noscript>
  Please enable JavaScript to view examples.
</noscript>

> To show the browser in the sandbox, you may need to click `Add DevTools >
Other Previews > 8080.`

<template>
  <iframe src="https://codesandbox.io/p/devbox/1-basic-component-0-7-qvgdxs?file=%2Fsrc%2Fmain.rs%3A1%2C1-59%2C2&workspaceId=478437f3-1f86-4b1e-b665-5c27a31451fb" width="100%" height="1000px" style="max-height: 100vh"></iframe>
</template>

```

<details>
<summary>CodeSandbox Source</summary>

```rust
use leptos::prelude::*;

// The #[component] macro marks a function as a reusable component
// Components are the building blocks of your user interface
// They define a reusable unit of behavior
#[component]
fn App() -> impl IntoView {
    // here we create a reactive signal
    // and get a (getter, setter) pair
    // signals are the basic unit of change in the framework
    // we'll talk more about them later
    let (count, set_count) = signal(0);

    // the `view` macro is how we define the user interface
    // it uses an HTML-like format that can accept certain Rust values
    view! {
        <button
            // on:click will run whenever the `click` event fires
            // every event handler is defined as `on:{eventname}`

            // we're able to move `set_count` into the closure
            // because signals are Copy and 'static

            on:click=move |_| *set_count.write() += 1
        >
            // text nodes in RSX should be wrapped in quotes,
            // like a normal Rust string
            "Click me: "
            {count}
        </button>
        <p>
            <strong>"Reactive: "</strong>
            // you can insert Rust expressions as values in the DOM
            // by wrapping them in curly braces
            // if you pass in a function, it will reactively update
            {move || count.get()}
        </p>
        <p>
            <strong>"Reactive shorthand: "</strong>
            // you can use signals directly in the view, as a shorthand
            // for a function that just wraps the getter
            {count}
        </p>
        <p>
            <strong>"Not reactive: "</strong>
            // NOTE: if you just write {count.get()}, this will *not* be reactive
            // it simply gets the value of count once
            {count.get()}
        </p>
    }
}

// This `main` function is the entry point into the app
// It just mounts our component to the <body>
// Because we defined it as `fn App`, we can now use it in a
// template as <App/>
fn main() {
    leptos::mount::mount_to_body(App)
}
```
</details>
