# Loading Data with Resources

A [Resource](https://docs.rs/leptos/latest/leptos/struct.Resource.html) is a reactive data structure that reflects the current state of an asynchronous task, allowing you to integrate asynchronous `Future`s into the synchronous reactive system. 

Resources effectively allow you to load some async data, and then reactively access it either synchronously or asynchronously. You can `.await` them like ordinary `Future`s. But you can also access them with `.get()` and other signal access methods, as if a resource were a signal that returns `Some(T)` if it has resolved, and `None` if it’s still pending.

You do this by using [`Resource::new`](https://docs.rs/leptos/latest/leptos/struct.Resource.html). This takes two arguments:

1. a source signal, which will generate a new `Future` whenever it changes
2. a fetcher function, which takes the data from that signal and returns a `Future`

Here’s an example

```rust
// our source signal: some synchronous, local state
let (count, set_count) = signal(0);

// our resource
let async_data = Resource::new(
    move || count.get(),
    // every time `count` changes, this will run
    |value| async move {
        logging::log!("loading data from API");
        load_data(value).await
    },
);
```

To create a resource that simply runs once, you can pass a non-reactive, empty source signal:

```rust
let once = Resource::new(|| (), |_| async move { load_data().await });
```

To access the value you can use `.get()` (or `.read()` or `.with()`). These work just like `.get()` and friends on a signal, but they always return `Option<T>`, not `T`: because it’s always possible that your resource is still loading.

So, you can show the current state of a resource in your view:

```rust
let once = Resource::new(|| (), |_| async move { load_data().await });
view! {
    <h1>"My Data"</h1>
    {move || match once.get() {
        None => view! { <p>"Loading..."</p> }.into_view(),
        Some(data) => view! { <ShowData data/> }.into_view()
    }}
}
```

Resources also provide a `refetch()` method that allows you to manually reload the data (for example, in response to a button click). 

```admonish sandbox title="Live example" collapsible=true

[Click to open CodeSandbox.](https://codesandbox.io/p/sandbox/10-resources-0-5-x6h5j6?file=%2Fsrc%2Fmain.rs%3A2%2C3)

<noscript>
  Please enable JavaScript to view examples.
</noscript>

<template>
  <iframe src="https://codesandbox.io/p/sandbox/10-resources-0-5-9jq86q?file=%2Fsrc%2Fmain.rs%3A2%2C3" width="100%" height="1000px" style="max-height: 100vh"></iframe>
</template>

```

<details>
<summary>CodeSandbox Source</summary>

```rust
use gloo_timers::future::TimeoutFuture;
use leptos::*;

// Here we define an async function
// This could be anything: a network request, database read, etc.
// Here, we just multiply a number by 10
async fn load_data(value: i32) -> i32 {
    // fake a one-second delay
    TimeoutFuture::new(1_000).await;
    value * 10
}

#[component]
fn App() -> impl IntoView {
    // this count is our synchronous, local state
    let (count, set_count) = create_signal(0);

    // create_resource takes two arguments after its scope
    let async_data = create_resource(
        // the first is the "source signal"
        count,
        // the second is the loader
        // it takes the source signal's value as its argument
        // and does some async work
        |value| async move { load_data(value).await },
    );
    // whenever the source signal changes, the loader reloads

    // you can also create resources that only load once
    // just return the unit type () from the source signal
    // that doesn't depend on anything: we just load it once
    let stable = create_resource(|| (), |_| async move { load_data(1).await });

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

    // the resource's loading() method gives us a
    // signal to indicate whether it's currently loading
    let loading = async_data.loading();
    let is_loading = move || if loading() { "Loading..." } else { "Idle." };

    view! {
        <button
            on:click=move |_| {
                set_count.update(|n| *n += 1);
            }
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
            {is_loading}
        </p>
    }
}

fn main() {
    leptos::mount_to_body(App)
}
```

</details>
</preview>
