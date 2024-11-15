# `<Suspense/>`

In the previous chapter, we showed how you can create a simple loading screen to show some fallback while a resource is loading.

```rust
let (count, set_count) = signal(0);
let once = Resource::new(move || count.get(), |count| async move { load_a(count).await });

view! {
    <h1>"My Data"</h1>
    {move || match once.get() {
        None => view! { <p>"Loading..."</p> }.into_view(),
        Some(data) => view! { <ShowData data/> }.into_view()
    }}
}
```

But what if we have two resources, and want to wait for both of them?

```rust
let (count, set_count) = signal(0);
let (count2, set_count2) = signal(0);
let a = Resource::new(move || count.get(), |count| async move { load_a(count).await });
let b = Resource::new(move || count2.get(), |count| async move { load_b(count).await });

view! {
    <h1>"My Data"</h1>
    {move || match (a.get(), b.get()) {
        (Some(a), Some(b)) => view! {
            <ShowA a/>
            <ShowA b/>
        }.into_view(),
        _ => view! { <p>"Loading..."</p> }.into_view()
    }}
}
```

That’s not _so_ bad, but it’s kind of annoying. What if we could invert the flow of control?

The [`<Suspense/>`](https://docs.rs/leptos/latest/leptos/fn.Suspense.html) component lets us do exactly that. You give it a `fallback` prop and children, one or more of which usually involves reading from a resource. Reading from a resource “under” a `<Suspense/>` (i.e., in one of its children) registers that resource with the `<Suspense/>`. If it’s still waiting for resources to load, it shows the `fallback`. When they’ve all loaded, it shows the children.

```rust
let (count, set_count) = signal(0);
let (count2, set_count2) = signal(0);
let a = Resource::new(count, |count| async move { load_a(count).await });
let b = Resource::new(count2, |count| async move { load_b(count).await });

view! {
    <h1>"My Data"</h1>
    <Suspense
        fallback=move || view! { <p>"Loading..."</p> }
    >
        <h2>"My Data"</h2>
        <h3>"A"</h3>
        {move || {
            a.get()
                .map(|a| view! { <ShowA a/> })
        }}
        <h3>"B"</h3>
        {move || {
            b.get()
                .map(|b| view! { <ShowB b/> })
        }}
    </Suspense>
}
```

Every time one of the resources is reloading, the `"Loading..."` fallback will show again.

This inversion of the flow of control makes it easier to add or remove individual resources, as you don’t need to handle the matching yourself. It also unlocks some massive performance improvements during server-side rendering, which we’ll talk about during a later chapter.

Using `<Suspense/>` also gives us access to a useful way to directly `.await` resources, allowing us to remove a level of nesting, above. The `Suspend` type lets us create a renderable `Future` which can be used in the view:

```rust
view! {
    <h1>"My Data"</h1>
    <Suspense
        fallback=move || view! { <p>"Loading..."</p> }
    >
        <h2>"My Data"</h2>
        {move || Suspend::new(async move {
            let a = a.await;
            let b = b.await;
            view! {
                <h3>"A"</h3>
                <ShowA a/>
                <h3>"B"</h3>
                <ShowB b/>
            }
        })}
    </Suspense>
}
```

`Suspend` allows us to avoid null-checking each resource, and removes some additional complexity from the code.

## `<Await/>`

If you’re simply trying to wait for some `Future` to resolve before rendering, you may find the `<Await/>` component helpful in reducing boilerplate. `<Await/>` essentially combines a `OnceResource` with a `<Suspense/>` with no fallback.

In other words:

1. It only polls the `Future` once, and does not respond to any reactive changes.
2. It does not render anything until the `Future` resolves.
3. After the `Future` resolves, it binds its data to whatever variable name you choose and then renders its children with that variable in scope.

```rust
async fn fetch_monkeys(monkey: i32) -> i32 {
    // maybe this didn't need to be async
    monkey * 2
}
view! {
    <Await
        // `future` provides the `Future` to be resolved
        future=fetch_monkeys(3)
        // the data is bound to whatever variable name you provide
        let:data
    >
        // you receive the data by reference and can use it in your view here
        <p>{*data} " little monkeys, jumping on the bed."</p>
    </Await>
}
```

```admonish sandbox title="Live example" collapsible=true

[Click to open CodeSandbox.](https://codesandbox.io/p/devbox/11-suspense-0-7-sr2srk?file=%2Fsrc%2Fmain.rs%3A1%2C1-55%2C1)

<noscript>
  Please enable JavaScript to view examples.
</noscript>

<template>
  <iframe src="https://codesandbox.io/p/devbox/11-suspense-0-7-sr2srk?file=%2Fsrc%2Fmain.rs%3A1%2C1-55%2C1" width="100%" height="1000px" style="max-height: 100vh"></iframe>
</template>

```

<details>
<summary>CodeSandbox Source</summary>

```rust
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;

async fn important_api_call(name: String) -> String {
    TimeoutFuture::new(1_000).await;
    name.to_ascii_uppercase()
}

#[component]
pub fn App() -> impl IntoView {
    let (name, set_name) = signal("Bill".to_string());

    // this will reload every time `name` changes
    let async_data = LocalResource::new(move || important_api_call(name.get()));

    view! {
        <input
            on:change:target=move |ev| {
                set_name.set(ev.target().value());
            }
            prop:value=name
        />
        <p><code>"name:"</code> {name}</p>
        <Suspense
            // the fallback will show whenever a resource
            // read "under" the suspense is loading
            fallback=move || view! { <p>"Loading..."</p> }
        >
            // Suspend allows you use to an async block in the view
            <p>
                "Your shouting name is "
                {move || Suspend::new(async move {
                    async_data.await
                })}
            </p>
        </Suspense>
        <Suspense
            // the fallback will show whenever a resource
            // read "under" the suspense is loading
            fallback=move || view! { <p>"Loading..."</p> }
        >
            // the children will be rendered once initially,
            // and then whenever any resources has been resolved
            <p>
                "Which should be the same as... "
                {move || async_data.get().as_deref().map(ToString::to_string)}
            </p>
        </Suspense>
    }
}

fn main() {
    leptos::mount::mount_to_body(App)
}
```

</details>
</preview>
