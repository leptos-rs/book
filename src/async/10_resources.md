# Loading Data with Resources

Resources are reactive wrappers for asynchronous tasks, which allow you to integrate an asynchronous `Future` into the synchronous reactive system.

They effectively allow you to load some async data, and then reactively access it either synchronously or asynchronously. You can `.await` a resource like an ordinary `Future`, and this will track it. But you can also access a resource with `.get()` and other signal access methods, as if a resource were a signal that returns `Some(T)` if it has resolved, and `None` if it’s still pending.

Resources come in two primary flavors: `Resource` and `LocalResource`. If you’re using server-side rendering (which this book will discuss later), you should default to using `Resource`. If you’re using client-side rendering with a `!Send` API (like many of the browser APIs), or if you are using SSR but have some async task that can only be done on the browser (for example, accessing an async browser API) then you should use `LocalResource`.

## Local Resources

`LocalResource::new()` takes a single argument: a “fetcher” function that returns a `Future`.

The `Future` can be an `async` block, the result of an `async fn` call, or any other Rust `Future`. The function will work like a derived signal or the other reactive closures that we’ve seen so far: you can read signals inside it, and whenever the signal changes, the function will run again, creating a new `Future` to run.

```rust
// this count is our synchronous, local state
let (count, set_count) = signal(0);

// tracks `count`, and reloads by calling `load_data`
// whenever it changes
let async_data = LocalResource::new(move || load_data(count.get()));
```

Creating a resource immediately calls its fetcher and begins polling the `Future`. Reading from a resource will return `None` until the async task completes, at which point it will notify its subscribers, and now have `Some(value)`.

You can also `.await` a resource. This might seem pointless—Why would you create a wrapper around a `Future`, only to then `.await` it? We’ll see why in the next chapter.

## Resources

If you’re using SSR, you should be using `Resource` instead of `LocalResource` in most cases.

This API is slightly different. `Resource::new()` takes two functions as its arguments:

1. a source function, which contains the “input.” This input is memoized, and whenever its value changes, the fetcher will be called.
2. a fetcher function, which takes the data from the source function and returns a `Future`

Unlike a `LocalResource`, a `Resource` serializes its value from the server to the client. Then, on the client, when first loading the page, the initial value will be deserialized rather than the async task running again. This is extremely important and very useful: It means that rather than waiting for the client WASM bundle to load and begin running the application, data loading begins on the server. (There will be more to say about this in later chapters.)

This is also why the API is split into two parts: signals in the _source_ function are tracked, but signals in the _fetcher_ are untracked, because this allows the resource to maintain reactivity without needing to run the fetcher again during initial hydration on the client.

Here’s the same example, using `Resource` instead of `LocalResource`

```rust
// this count is our synchronous, local state
let (count, set_count) = signal(0);

// our resource
let async_data = Resource::new(
    move || count.get(),
    // every time `count` changes, this will run
    |count| load_data(count)
);
```

Resources also provide a `refetch()` method that allows you to manually reload the data (for example, in response to a button click).

To create a resource that simply runs once, you can use `OnceResource`, which simply takes a `Future`, and adds some optimizations that come from knowing it will only load once.

```rust
let once = OnceResource::new(load_data(42));
```

## Accessing Resources

Both `LocalResource` and `Resource` implement the various signal access methods (`.read()`, `.with()`, `.get()`), but return `Option<T>` instead of `T`; they will be `None` until the async data has loaded.

```admonish sandbox title="Live example" collapsible=true

[Click to open CodeSandbox.](https://codesandbox.io/p/devbox/10-resource-0-7-q5xr9m?file=%2Fsrc%2Fmain.rs%3A7%2C30)

<noscript>
  Please enable JavaScript to view examples.
</noscript>

<template>
  <iframe src="https://codesandbox.io/p/devbox/10-resource-0-7-q5xr9m?file=%2Fsrc%2Fmain.rs%3A7%2C30" width="100%" height="1000px" style="max-height: 100vh"></iframe>
</template>

```

<details>
<summary>CodeSandbox Source</summary>

```rust
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;

// Here we define an async function
// This could be anything: a network request, database read, etc.
// Here, we just multiply a number by 10
async fn load_data(value: i32) -> i32 {
    // fake a one-second delay
    TimeoutFuture::new(1_000).await;
    value * 10
}

#[component]
pub fn App() -> impl IntoView {
    // this count is our synchronous, local state
    let (count, set_count) = signal(0);

    // tracks `count`, and reloads by calling `load_data`
    // whenever it changes
    let async_data = LocalResource::new(move || load_data(count.get()));

    // a resource will only load once if it doesn't read any reactive data
    let stable = LocalResource::new(|| load_data(1));

    // we can access the resource values with .get()
    // this will reactively return None before the Future has resolved
    // and update to Some(T) when it has resolved
    let async_result = move || {
        async_data
            .get()
            .map(|value| format!("Server returned {value:?}"))
            // This loading state will only show before the first load
            .unwrap_or_else(|| "Loading...".into())
    };

    view! {
        <button
            on:click=move |_| *set_count.write() += 1
        >
            "Click me"
        </button>
        <p>
            <code>"stable"</code>": " {move || stable.get()}
        </p>
        <p>
            <code>"count"</code>": " {count}
        </p>
        <p>
            <code>"async_value"</code>": "
            {async_result}
            <br/>
        </p>
    }
fn main() {
    leptos::mount::mount_to_body(App)
}
```

</details>
</preview>
