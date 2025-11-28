# Server Functions

If you’re creating anything beyond a toy app, you’ll need to run code on the server all the time: reading from or writing to a database that only runs on the server, running expensive computations using libraries you don’t want to ship down to the client, accessing APIs that need to be called from the server rather than the client for CORS reasons or because you need a secret API key that’s stored on the server and definitely shouldn’t be shipped down to a user’s browser.

Traditionally, this is done by separating your server and client code, and by setting up something like a REST API or GraphQL API to allow your client to fetch and mutate data on the server. This is fine, but it requires you to write and maintain your code in multiple separate places (client-side code for fetching, server-side functions to run), as well as creating a third thing to manage, which is the API contract between the two.

Leptos is one of a number of modern frameworks that introduce the concept of **server functions**. Server functions have two key characteristics:

1. Server functions are **co-located** with your component code, so that you can organize your work by feature, not by technology. For example, you might have a “dark mode” feature that should persist a user’s dark/light mode preference across sessions, and be applied during server rendering so there’s no flicker. This requires a component that needs to be interactive on the client, and some work to be done on the server (setting a cookie, maybe even storing a user in a database.) Traditionally, this feature might end up being split between two different locations in your code, one in your “frontend” and one in your “backend.” With server functions, you’ll probably just write them both in one `dark_mode.rs` and forget about it.
2. Server functions are **isomorphic**, i.e., they can be called either from the server or the browser. This is done by generating code differently for the two platforms. On the server, a server function simply runs. In the browser, the server function’s body is replaced with a stub that actually makes a fetch request to the server, serializing the arguments into the request and deserializing the return value from the response. But on either end, the function can simply be called: you can create an `add_todo` function that writes to your database, and simply call it from a click handler on a button in the browser!

## Using Server Functions

Actually, I kind of like that example. What would it look like? It’s pretty simple, actually.

```rust
// todo.rs

#[server]
pub async fn add_todo(title: String) -> Result<(), ServerFnError> {
    let mut conn = db().await?;

    match sqlx::query("INSERT INTO todos (title, completed) VALUES ($1, false)")
        .bind(title)
        .execute(&mut conn)
        .await
    {
        Ok(_row) => Ok(()),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

#[component]
pub fn BusyButton() -> impl IntoView {
	view! {
        <button on:click=move |_| {
            spawn_local(async {
                add_todo("So much to do!".to_string()).await;
            });
        }>
            "Add Todo"
        </button>
	}
}
```

You’ll notice a couple things here right away:

- Server functions can use server-only dependencies, like `sqlx`, and can access server-only resources, like our database.
- Server functions are `async`. Even if they only did synchronous work on the server, the function signature would still need to be `async`, because calling them from the browser _must_ be asynchronous.
- Server functions return `Result<T, ServerFnError>`. Again, even if they only do infallible work on the server, this is true, because `ServerFnError`’s variants include the various things that can be wrong during the process of making a network request.
- Server functions can be called from the client. Take a look at our click handler. This is code that will _only ever_ run on the client. But it can call the function `add_todo` (using `spawn_local` to run the `Future`) as if it were an ordinary async function:

```rust
move |_| {
	spawn_local(async {
		add_todo("So much to do!".to_string()).await;
	});
}
```

- Server functions are top-level functions defined with `fn`. Unlike event listeners, derived signals, and most everything else in Leptos, they are not closures! As `fn` calls, they have no access to the reactive state of your app or anything else that is not passed in as an argument. And again, this makes perfect sense: When you make a request to the server, the server doesn’t have access to client state unless you send it explicitly. (Otherwise we’d have to serialize the whole reactive system and send it across the wire with every request. This would not be a great idea.)
- Server function arguments and return values both need to be serializable. Again, hopefully this makes sense: while function arguments in general don’t need to be serialized, calling a server function from the browser means serializing the arguments and sending them over HTTP.

There are a few things to note about the way you define a server function, too.

- Server functions are created by using the [`#[server]` macro](https://docs.rs/leptos/latest/leptos/attr.server.html) to annotate a top-level function, which can be defined anywhere.

Server functions work by using conditional compilation. On the server, the server function creates an HTTP endpoint that receives its arguments as an HTTP request, and returns its result as an HTTP response. For the client-side/browser build, the body of the server function is stubbed out with an HTTP request.

```admonish warning
### An Important Note about Security

Server functions are a cool technology, but it’s very important to remember. **Server functions are not magic; they’re syntax sugar for defining a public API.** The _body_ of a server function is never made public; it’s just part of your server binary. But the server function is a publicly accessible API endpoint, and its return value is just a JSON or similar blob. Do not return information from a server function unless it is public, or you've implemented proper security procedures. These procedures might include authenticating incoming requests, ensuring proper encryption, rate limiting access, and more.
```

## Customizing Server Functions

By default, server functions encode their arguments as an HTTP POST request (using `serde_qs`) and their return values as JSON (using `serde_json`). This default is intended to promote compatibility with the `<form>` element, which has native support for making POST requests, even when WASM is disabled, unsupported, or has not yet loaded. They mount their endpoints at a hashed URL intended to prevent name collisions.

However, there are many ways to customize server functions, with a variety of supported input and output encodings, the ability to set specific endpoints, and so on.

Take a look at the docs for the [`#[server]` macro](https://docs.rs/leptos/latest/leptos/attr.server.html) and [`server_fn` crate](https://docs.rs/server_fn/latest/server_fn/), and the extensive [`server_fns_axum` example](https://github.com/leptos-rs/leptos/blob/main/examples/server_fns_axum/src/app.rs) in the repo for more information and examples.

## Using Custom Errors

Server functions can return any kind of errors that implement the `FromServerFnError` trait.
This makes error handling much more ergonomic and allows you to provide domain-specific error information to your clients:

```rust
use leptos::prelude::*;
use server_fn::codec::JsonEncoding;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AppError {
    ServerFnError(ServerFnErrorErr),
    DbError(String),
}

impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        AppError::ServerFnError(value)
    }
}

#[server]
pub async fn create_user(name: String, email: String) -> Result<User, AppError> {
    // Try to create user in database
    match insert_user_into_db(&name, &email).await {
        Ok(user) => Ok(user),
        Err(e) => Err(AppError::DbError(e.to_string())),
    }
}
```

## Quirks to Note

Server functions come with a few quirks that are worth noting:

- Using pointer-sized integer types such as `isize` and `usize` can lead to errors when making calls between the 32-bit WASM architecture and a 64-bit server architecture; if the server responds with a value that doesn't fit in 32 bits, this will lead to a deserialization error. Use fixed size types such as `i32` or `i64` to mitigate this problem.
- Arguments sent to the server are URL-encoded using `serde_qs` by default. This allows them to work well with `<form>` elements, but can have some quirks: for example, the current version of `serde_qs` does not always work well with optional types (see [here](https://github.com/leptos-rs/leptos/issues/3832) or [here](https://github.com/leptos-rs/leptos/issues/4016)) or with enums that have tuple variants (see [here](https://github.com/leptos-rs/leptos/issues/4464)). You can use the workarounds described in those issues, or [switch to an alternate input encoding](https://docs.rs/leptos/latest/leptos/attr.server.html#named-arguments).

## Integrating Server Functions with Leptos

So far, everything I’ve said is actually framework agnostic. (And in fact, the Leptos server function crate has been integrated into Dioxus as well!) Server functions are simply a way of defining a function-like RPC call that leans on Web standards like HTTP requests and URL encoding.

But in a way, they also provide the last missing primitive in our story so far. Because a server function is just a plain Rust async function, it integrates perfectly with the async Leptos primitives we discussed [earlier](../async/index.html). So you can easily integrate your server functions with the rest of your applications:

- Create **resources** that call the server function to load data from the server
- Read these resources under `<Suspense/>` or `<Transition/>` to enable streaming SSR and fallback states while data loads.
- Create **actions** that call the server function to mutate data on the server

The final section of this book will make this a little more concrete by introducing patterns that use progressively-enhanced HTML forms to run these server actions.

But in the next few chapters, we’ll actually take a look at some of the details of what you might want to do with your server functions, including the best ways to integrate with the powerful extractors provided by the Actix and Axum server frameworks.
