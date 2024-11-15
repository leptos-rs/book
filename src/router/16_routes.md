# Defining Routes

## Getting Started

It’s easy to get started with the router.

First things first, make sure you’ve added the `leptos_router` package to your dependencies. Unlike `leptos`, this does not have separate `csr` and `hydrate` features; it does have an `ssr` feature, intended for use only on the server side, so activate that for your server-side build.


> It’s important that the router is a separate package from `leptos` itself. This means that everything in the router can be defined in user-land code. If you want to create your own router, or use no router, you’re completely free to do that!

And import the relevant types from the router, either with something like

```rust
use leptos_router::components::{Router, Route, Routes};
```

## Providing the `<Router/>`

Routing behavior is provided by the [`<Router/>`](https://docs.rs/leptos_router/latest/leptos_router/fn.Router.html) component. This should usually be somewhere near the root of your application, wrapping the rest of the app.

> You shouldn’t try to use multiple `<Router/>`s in your app. Remember that the router drives global state: if you have multiple routers, which one decides what to do when the URL changes?

Let’s start with a simple `<App/>` component using the router:

```rust
use leptos::prelude::*;
use leptos_router::components::Router;

#[component]
pub fn App() -> impl IntoView {
    view! {
      <Router>
        <nav>
          /* ... */
        </nav>
        <main>
          /* ... */
        </main>
      </Router>
    }
}

```

## Defining `<Routes/>`

The [`<Routes/>`](https://docs.rs/leptos_router/latest/leptos_router/fn.Routes.html) component is where you define all the routes to which a user can navigate in your application. Each possible route is defined by a [`<Route/>`](https://docs.rs/leptos_router/latest/leptos_router/fn.Route.html) component.

You should place the `<Routes/>` component at the location within your app where you want routes to be rendered. Everything outside `<Routes/>` will be present on every page, so you can leave things like a navigation bar or menu outside the `<Routes/>`.

```rust
use leptos::prelude::*;
use leptos_router::components::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
      <Router>
        <nav>
          /* ... */
        </nav>
        <main>
          // all our routes will appear inside <main>
          <Routes fallback=|| "Not found.">
            /* ... */
          </Routes>
        </main>
      </Router>
    }
}
```

`<Routes/>` should also have a `fallback`, a function that defines what should be shown if no route is matched.

Individual routes are defined by providing children to `<Routes/>` with the `<Route/>` component. `<Route/>` takes a `path` and a `view`. When the current location matches `path`, the `view` will be created and displayed.

The `path` is most easily defined using the `path` macro, and can include

- a static path (`/users`),
- dynamic, named parameters beginning with a colon (`/:id`),
- and/or a wildcard beginning with an asterisk (`/user/*any`)

The `view` is a function that returns a view. Any component with no props works here, as does a closure that returns some view.

```rust
<Routes fallback=|| "Not found.">
  <Route path=path!("/") view=Home/>
  <Route path=path!("/users") view=Users/>
  <Route path=path!("/users/:id") view=UserProfile/>
  <Route path=path!("/*any") view=|| view! { <h1>"Not Found"</h1> }/>
</Routes>
```

> `view` takes a `Fn() -> impl IntoView`. If a component has no props, it can be passed directly into the `view`. In this case, `view=Home` is just a shorthand for `|| view! { <Home/> }`.

Now if you navigate to `/` or to `/users` you’ll get the home page or the `<Users/>`. If you go to `/users/3` or `/blahblah` you’ll get a user profile or your 404 page (`<NotFound/>`). On every navigation, the router determines which `<Route/>` should be matched, and therefore what content should be displayed where the `<Routes/>` component is defined.

Simple enough?
