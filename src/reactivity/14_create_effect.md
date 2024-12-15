# Responding to Changes with Effects

We’ve made it this far without having mentioned half of the reactive system: effects.

Reactivity works in two halves: updating individual reactive values (“signals”) notifies the pieces of code that depend on them (“effects”) that they need to run again. These two halves of the reactive system are inter-dependent. Without effects, signals can change within the reactive system but never be observed in a way that interacts with the outside world. Without signals, effects run once but never again, as there’s no observable value to subscribe to. Effects are quite literally “side effects” of the reactive system: they exist to synchronize the reactive system with the non-reactive world outside it.

The renderer uses effects to update parts of the DOM in response to changes in signals. You can create your own effects to synchronize the reactive system with the outside world in other ways.

[`Effect::new`](https://docs.rs/leptos/0.7.0-gamma3/leptos/reactive/effect/struct.Effect.html) takes a function as its argument. It runs this function on the next “tick” of the reactive system. (So for example, if you use it in a component, it will run just _after_ that component has been rendered.) If you access any reactive signal inside that function, it registers the fact that the effect depends on that signal. Whenever one of the signals that the effect depends on changes, the effect runs again.

```rust
let (a, set_a) = signal(0);
let (b, set_b) = signal(0);

Effect::new(move |_| {
  // immediately prints "Value: 0" and subscribes to `a`
  logging::log!("Value: {}", a.get());
});
```

The effect function is called with an argument containing whatever value it returned the last time it ran. On the initial run, this is `None`.

By default, effects **do not run on the server**. This means you can call browser-specific APIs within the effect function without causing issues. If you need an effect to run on the server, use [`Effect::new_isomorphic`](https://docs.rs/leptos/0.7.0-gamma3/leptos/reactive/effect/struct.Effect.html#method.new_isomorphic).

## Auto-tracking and Dynamic Dependencies

If you’re familiar with a framework like React, you might notice one key difference. React and similar frameworks typically require you to pass a “dependency array,” an explicit set of variables that determine when the effect should rerun.

Because Leptos comes from the tradition of synchronous reactive programming, we don’t need this explicit dependency list. Instead, we automatically track dependencies depending on which signals are accessed within the effect.

This has two effects (no pun intended). Dependencies are:

1. **Automatic**: You don’t need to maintain a dependency list, or worry about what should or shouldn’t be included. The framework simply tracks which signals might cause the effect to rerun, and handles it for you.
2. **Dynamic**: The dependency list is cleared and updated every time the effect runs. If your effect contains a conditional (for example), only signals that are used in the current branch are tracked. This means that effects rerun the absolute minimum number of times.

> If this sounds like magic, and if you want a deep dive into how automatic dependency tracking works, [check out this video](https://www.youtube.com/watch?v=GWB3vTWeLd4). (Apologies for the low volume!)

## Effects as Zero-Cost-ish Abstraction

While they’re not a “zero-cost abstraction” in the most technical sense—they require some additional memory use, exist at runtime, etc.—at a higher level, from the perspective of whatever expensive API calls or other work you’re doing within them, effects are a zero-cost abstraction. They rerun the absolute minimum number of times necessary, given how you’ve described them.

Imagine that I’m creating some kind of chat software, and I want people to be able to display their full name, or just their first name, and to notify the server whenever their name changes:

```rust
let (first, set_first) = signal(String::new());
let (last, set_last) = signal(String::new());
let (use_last, set_use_last) = signal(true);

// this will add the name to the log
// any time one of the source signals changes
Effect::new(move |_| {
    logging::log!(
        "{}", if use_last.get() {
            format!("{} {}", first.get(), last.get())
        } else {
            first.get()
        },
    )
});
```

If `use_last` is `true`, effect should rerun whenever `first`, `last`, or `use_last` changes. But if I toggle `use_last` to `false`, a change in `last` will never cause the full name to change. In fact, `last` will be removed from the dependency list until `use_last` toggles again. This saves us from sending multiple unnecessary requests to the API if I change `last` multiple times while `use_last` is still `false`.

## To create an effect, or not to create an effect?

Effects are intended to synchronize the reactive system with the non-reactive world outside, not to synchronize between different reactive values. In other words: using an effect to read a value from one signal and set it in another is always sub-optimal.

If you need to define a signal that depends on the value of other signals, use a derived signal or a [`Memo`](https://docs.rs/leptos/0.7.0-gamma3/leptos/reactive/computed/struct.Memo.html). Writing to a signal inside an effect isn’t the end of the world, and it won’t cause your computer to light on fire, but a derived signal or memo is always better—not only because the dataflow is clear, but because the performance is better.

```rust
let (a, set_a) = signal(0);

// ⚠️ not great
let (b, set_b) = signal(0);
Effect::new(move |_| {
    set_b.set(a.get() * 2);
});

// ✅ woo-hoo!
let b = move || a.get() * 2;
```

If you need to synchronize some reactive value with the non-reactive world outside—like a web API, the console, the filesystem, or the DOM—writing to a signal in an effect is a fine way to do that. In many cases, though, you’ll find that you’re really writing to a signal inside an event listener or something else, not inside an effect. In these cases, you should check out [`leptos-use`](https://leptos-use.rs/) to see if it already provides a reactive wrapping primitive to do that!

> If you’re curious for more information about when you should and shouldn’t use `create_effect`, [check out this video](https://www.youtube.com/watch?v=aQOFJQ2JkvQ) for a more in-depth consideration!

## Effects and Rendering

We’ve managed to get this far without mentioning effects because they’re built into the Leptos DOM renderer. We’ve seen that you can create a signal and pass it into the `view` macro, and it will update the relevant DOM node whenever the signal changes:

```rust
let (count, set_count) = signal(0);

view! {
    <p>{count}</p>
}
```

This works because the framework essentially creates an effect wrapping this update. You can imagine Leptos translating this view into something like this:

```rust
let (count, set_count) = signal(0);

// create a DOM element
let document = leptos::document();
let p = document.create_element("p").unwrap();

// create an effect to reactively update the text
Effect::new(move |prev_value| {
    // first, access the signal’s value and convert it to a string
    let text = count.get().to_string();

    // if this is different from the previous value, update the node
    if prev_value != Some(text) {
        p.set_text_content(&text);
    }

    // return this value so we can memoize the next update
    text
});
```

Every time `count` is updated, this effect will rerun. This is what allows reactive, fine-grained updates to the DOM.

## Explicit Tracking with `Effect::watch()`

In addition to `Effect::new()`, Leptos provides an [`Effect::watch()`](https://docs.rs/leptos/0.7.0-gamma3/leptos/reactive/effect/struct.Effect.html#method.watch) function, which can be used to separate tracking and responding to changes by explicitly passing in a set of values to track.

`watch` takes a first argument, which is reactively tracked, and a second, which is not. Whenever a reactive value in its `deps` argument is changed, the `callback` is run. `watch` returns an `Effect`, which can be called with `.stop()` to stop tracking the dependencies.

```rust
let (num, set_num) = signal(0);

let effect = Effect::watch(
    move || num.get(),
    move |num, prev_num, _| {
        leptos::logging::log!("Number: {}; Prev: {:?}", num, prev_num);
    },
    false,
);

set_num.set(1); // > "Number: 1; Prev: Some(0)"

effect.stop(); // stop watching

set_num.set(2); // (nothing happens)
```

```admonish sandbox title="Live example" collapsible=true

[Click to open CodeSandbox.](https://codesandbox.io/p/devbox/14-effect-0-7-fxpy2d?file=%2Fsrc%2Fmain.rs%3A21%2C28&workspaceId=478437f3-1f86-4b1e-b665-5c27a31451fb)

<noscript>
  Please enable JavaScript to view examples.
</noscript>

<template>
  <iframe src="https://codesandbox.io/p/devbox/14-effect-0-7-fxpy2d?file=%2Fsrc%2Fmain.rs%3A21%2C28&workspaceId=478437f3-1f86-4b1e-b665-5c27a31451fb" width="100%" height="1000px" style="max-height: 100vh"></iframe>
</template>

```

<details>
<summary>CodeSandbox Source</summary>

```rust
use leptos::html::Input;
use leptos::prelude::*;

#[derive(Copy, Clone)]
struct LogContext(RwSignal<Vec<String>>);

#[component]
fn App() -> impl IntoView {
    // Just making a visible log here
    // You can ignore this...
    let log = RwSignal::<Vec<String>>::new(vec![]);
    let logged = move || log.get().join("\n");

    // the newtype pattern isn't *necessary* here but is a good practice
    // it avoids confusion with other possible future `RwSignal<Vec<String>>` contexts
    // and makes it easier to refer to it
    provide_context(LogContext(log));

    view! {
        <CreateAnEffect/>
        <pre>{logged}</pre>
    }
}

#[component]
fn CreateAnEffect() -> impl IntoView {
    let (first, set_first) = signal(String::new());
    let (last, set_last) = signal(String::new());
    let (use_last, set_use_last) = signal(true);

    // this will add the name to the log
    // any time one of the source signals changes
    Effect::new(move |_| {
        log(if use_last.get() {
            let first = first.read();
            let last = last.read();
            format!("{first} {last}")
        } else {
            first.get()
        })
    });

    view! {
        <h1>
            <code>"create_effect"</code>
            " Version"
        </h1>
        <form>
            <label>
                "First Name"
                <input
                    type="text"
                    name="first"
                    prop:value=first
                    on:change:target=move |ev| set_first.set(ev.target().value())
                />
            </label>
            <label>
                "Last Name"
                <input
                    type="text"
                    name="last"
                    prop:value=last
                    on:change:target=move |ev| set_last.set(ev.target().value())
                />
            </label>
            <label>
                "Show Last Name"
                <input
                    type="checkbox"
                    name="use_last"
                    prop:checked=use_last
                    on:change:target=move |ev| set_use_last.set(ev.target().checked())
                />
            </label>
        </form>
    }
}

#[component]
fn ManualVersion() -> impl IntoView {
    let first = NodeRef::<Input>::new();
    let last = NodeRef::<Input>::new();
    let use_last = NodeRef::<Input>::new();

    let mut prev_name = String::new();
    let on_change = move |_| {
        log("      listener");
        let first = first.get().unwrap();
        let last = last.get().unwrap();
        let use_last = use_last.get().unwrap();
        let this_one = if use_last.checked() {
            format!("{} {}", first.value(), last.value())
        } else {
            first.value()
        };

        if this_one != prev_name {
            log(&this_one);
            prev_name = this_one;
        }
    };

    view! {
        <h1>"Manual Version"</h1>
        <form on:change=on_change>
            <label>"First Name" <input type="text" name="first" node_ref=first/></label>
            <label>"Last Name" <input type="text" name="last" node_ref=last/></label>
            <label>
                "Show Last Name" <input type="checkbox" name="use_last" checked node_ref=use_last/>
            </label>
        </form>
    }
}

fn log(msg: impl std::fmt::Display) {
    let log = use_context::<LogContext>().unwrap().0;
    log.update(|log| log.push(msg.to_string()));
}

fn main() {
    leptos::mount::mount_to_body(App)
}
```

</details>
</preview>
