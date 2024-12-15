# No Macros: The View Builder Syntax

> If you’re perfectly happy with the `view!` macro syntax described so far, you’re welcome to skip this chapter. The builder syntax described in this section is always available, but never required.

For one reason or another, many developers would prefer to avoid macros. Perhaps you don’t like the limited `rustfmt` support. (Although, you should check out [`leptosfmt`](https://github.com/bram209/leptosfmt), which is an excellent tool!) Perhaps you worry about the effect of macros on compile time. Perhaps you prefer the aesthetics of pure Rust syntax, or you have trouble context-switching between an HTML-like syntax and your Rust code. Or perhaps you want more flexibility in how you create and manipulate HTML elements than the `view` macro provides.

If you fall into any of those camps, the builder syntax may be for you.

The `view` macro expands an HTML-like syntax to a series of Rust functions and method calls. If you’d rather not use the `view` macro, you can simply use that expanded syntax yourself. And it’s actually pretty nice!

First off, if you want you can even drop the `#[component]` macro: a component is just a setup function that creates your view, so you can define a component as a simple function call:

```rust
pub fn counter(initial_value: i32, step: u32) -> impl IntoView { }
```

Elements are created by calling a function with the same name as the HTML element:

```rust
p()
```

You can add children to the element with [`.child()`](https://docs.rs/leptos/latest/leptos/html/trait.ElementChild.html#tymethod.child), which takes a single child or a tuple or array of types that implement [`IntoView`](https://docs.rs/leptos/latest/leptos/trait.IntoView.html).

```rust
p().child((em().child("Big, "), strong().child("bold "), "text"))
```

Attributes are added with [`.attr()`](https://docs.rs/leptos/latest/leptos/attr/custom/trait.CustomAttribute.html#method.attr). This can take any of the same types that you could pass as an attribute into the view macro (types that implement [`Attribute`](https://docs.rs/leptos/latest/leptos/attr/trait.Attribute.html)).

```rust
p().attr("id", "foo")
    .attr("data-count", move || count.get().to_string())
```

They can also be added with attribute methods, which are available for any built-in HTML attribute name:

```rust
p().id("foo")
    .attr("data-count", move || count.get().to_string())
```

Similarly, the `class:`, `prop:`, and `style:` syntaxes map directly onto [`.class()`](https://docs.rs/leptos/latest/leptos/attr/global/trait.ClassAttribute.html#tymethod.class), [`.prop()`](https://docs.rs/leptos/latest/leptos/attr/global/trait.PropAttribute.html#tymethod.prop), and [`.style()`](https://docs.rs/leptos/latest/leptos/attr/global/trait.StyleAttribute.html#tymethod.style) methods.

Event listeners can be added with [`.on()`](https://docs.rs/leptos/latest/leptos/attr/global/trait.OnAttribute.html#tymethod.on). Typed events found in [`leptos::ev`](https://docs.rs/leptos/latest/leptos/tachys/html/event/index.html) prevent typos in event names and allow for correct type inference in the callback function.

```rust
button()
    .on(ev::click, move |_| set_count.set(0))
    .child("Clear")
```

All of this adds up to a very Rusty syntax to build full-featured views, if you prefer this style.

```rust
/// A simple counter view.
// A component is really just a function call: it runs once to create the DOM and reactive system
pub fn counter(initial_value: i32, step: i32) -> impl IntoView {
    let (count, set_count) = signal(initial_value);
    div().child((
        button()
            // typed events found in leptos::ev
            // 1) prevent typos in event names
            // 2) allow for correct type inference in callbacks
            .on(ev::click, move |_| set_count.set(0))
            .child("Clear"),
        button()
            .on(ev::click, move |_| *set_count.write() -= step)
            .child("-1"),
        span().child(("Value: ", move || count.get(), "!")),
        button()
            .on(ev::click, move |_| *set_count.write() += step)
            .child("+1"),
    ))
}
```
