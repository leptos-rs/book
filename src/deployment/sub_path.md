# Deploying at Non-Root Paths

So far, the deployment steps have assumed that your application is deployed at the root path of your domain (`/`). However, it is also possible to deploy your application at a non-root path, such as `/my-app`.

If you are deploying at a non-root path, you’ll need to take a few steps to tell the various parts of the application what that new base path is.

## Update the Router `base`

The [`<Router/>`](https://docs.rs/leptos_router/latest/leptos_router/components/fn.Router.html) component has a `base` prop that specifies the base path for routing. For example, if you are deploying an application with three pages `/`, `/about`, and `/contact`, and you want them to be deployed at `/my-app` so that the three routes are `/my-app`, `/my-app/about`, and `/my-app/contact`, you would set the `base` prop to `/my-app`:

```rust
<Router base="/my-app">
    <Routes fallback=|| "Not found.">
        <Route path=path!("/") view=Home/>
        <Route path=path!("/about") view=About/>
        <Route path=path!("/contact") view=Contact/>
    </Routes>
</Router>
```

If you are using a reverse proxy, it’s likely that your server will *think* it’s serving `/` when it is actually serving `/my-app`. But in the browser, the router will still see the URL as `/my-app`. In this situation, you should set the `base` prop conditionally using conditional compilation:
```rust
let base = if cfg!(feature = "hydrate") {
    "/my-app"
} else {
    "/"
};
// ...
<Router base> // ...
```

## Update the `<HydrationScripts root/>`

If you’re using server rendering, the [`<HydrationScripts/>`](https://docs.rs/leptos/latest/leptos/hydration/fn.HydrationScripts.html) component is responsible for loading the JS/WASM to hydrate the app. This has its own `root` prop that specifies the base path for the hydration scripts. If they are also being served from a subdirectory, you should include that base path as the `root` prop.

## Update the Server Function URL

If you are using server functions, they will default to sending requests to `/`. If your server function handler is mounted at a different path, you can set that with [`set_server_url`](https://docs.rs/leptos/latest/leptos/server_fn/client/fn.set_server_url.html).

## Trunk configuration

If you’re using client-side rendering with Trunk, [consult the Trunk docs](https://trunkrs.dev/assets/#directives) on how to set the public URL via `--public-url`.
