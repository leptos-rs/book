# Interlude: Reactivity and Functions

One of our core contributors said to me recently: “I never used closures this often
until I started using Leptos.” And it’s true. Closures are at the heart of any Leptos
application. It sometimes looks a little silly:

```rust
// a signal holds a value, and can be updated
let (count, set_count) = signal(0);

// a derived signal is a function that accesses other signals
let double_count = move || count.get() * 2;
let count_is_odd = move || count.get() & 1 == 1;
let text = move || if count_is_odd() {
    "odd"
} else {
    "even"
};

// an effect automatically tracks the signals it depends on
// and reruns when they change
Effect::new(move |_| {
    logging::log!("text = {}", text());
});

view! {
    <p>{move || text().to_uppercase()}</p>
}
```

Closures, closures everywhere!

But why?

## Functions and UI Frameworks

Functions are at the heart of every UI framework. And this makes perfect sense. Creating a user interface is basically divided into two phases:

1. initial rendering
2. updates

In a web framework, the framework does some kind of initial rendering. Then it hands control back over to the browser. When certain events fire (like a mouse click) or asynchronous tasks finish (like an HTTP request finishing), the browser wakes the framework back up to update something. The framework runs some kind of code to update your user interface, and goes back asleep until the browser wakes it up again.

The key phrase here is “runs some kind of code.” The natural way to “run some kind of code” at an arbitrary point in time—in Rust or in any other programming language—is to call a function. And in fact every UI framework is based on rerunning some kind of function over and over:

1. virtual DOM (VDOM) frameworks like React, Yew, or Dioxus rerun a component or render function over and over, to generate a virtual DOM tree that can be reconciled with the previous result to patch the DOM
2. compiled frameworks like Angular and Svelte divide your component templates into “create” and “update” functions, rerunning the update function when they detect a change to the component’s state
3. in fine-grained reactive frameworks like SolidJS, Sycamore, or Leptos, _you_ define the functions that rerun

That’s what all our components are doing.

Take our typical `<SimpleCounter/>` example in its simplest form:

```rust
#[component]
pub fn SimpleCounter() -> impl IntoView {
    let (value, set_value) = signal(0);

    let increment = move |_| *set_value.write() += 1;

    view! {
        <button on:click=increment>
            {value}
        </button>
    }
}
```

The `SimpleCounter` function itself runs once. The `value` signal is created once. The framework hands off the `increment` function to the browser as an event listener. When you click the button, the browser calls `increment`, which updates `value` via `set_value`. And that updates the single text node represented in our view by `{value}`.

Functions are key to reactivity. They provide the framework with the ability to rerun the smallest possible unit of your application in response to a change.

So remember two things:

1. Your component function is a setup function, not a render function: it only runs once.
2. For values in your view template to be reactive, they must be reactive functions: either signals or closures that capture and read from signals.

```admonish note
This is actually the primary difference between the stable and nightly versions of Leptos. As you may know, using the nightly compiler and the `nightly` feature allows you to call a signal directly, as a function: so, `value()` instead of `value.get()`.

But this isn’t just syntax sugar. It allows for an extremely consistent semantic model: Reactive things are functions. Signals are accessed by calling functions. To say “give me a signal as an argument” you can take anything that `impl Fn() -> T`. And this function-based interface makes no distinction between signals, memos, and derived signals: any of them can be accessed by calling them as functions.

Unfortunately implementing the `Fn` traits on arbitrary structs like signals requires nightly Rust, although this particular feature has mostly just languished and is not likely to change (or be stabilized) any time soon. Many people avoid nightly, for one reason or another. So, over time we’ve moved the defaults for things like documentation toward stable. Unfortunately, this makes the simple mental model of “signals are functions” a bit less straightforward.
```
