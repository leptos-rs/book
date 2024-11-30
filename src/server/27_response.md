# Responses and Redirects

Extractors provide an easy way to access request data inside server functions. Leptos also provides a way to modify the HTTP response, using the `ResponseOptions` type (see docs for [Actix](https://docs.rs/leptos_actix/0.7.0-gamma3/leptos_actix/struct.ResponseOptions.html) or [Axum](https://docs.rs/leptos_axum/0.7.0-gamma3/leptos_axum/struct.ResponseOptions.html)) types and the `redirect` helper function (see docs for [Actix](https://docs.rs/leptos_actix/0.7.0-gamma3/leptos_actix/fn.redirect.html) or [Axum](https://docs.rs/leptos_axum/0.7.0-gamma3/leptos_axum/fn.redirect.html)).

## `ResponseOptions`

`ResponseOptions` is provided via context during the initial server rendering response and during any subsequent server function call. It allows you to easily set the status code for the HTTP response, or to add headers to the HTTP response, e.g., to set cookies.

```rust
#[server]
pub async fn tea_and_cookies() -> Result<(), ServerFnError> {
    use actix_web::{
        cookie::Cookie,
        http::header::HeaderValue,
        http::{header, StatusCode},
    };
    use leptos_actix::ResponseOptions;

    // pull ResponseOptions from context
    let response = expect_context::<ResponseOptions>();

    // set the HTTP status code
    response.set_status(StatusCode::IM_A_TEAPOT);

    // set a cookie in the HTTP response
    let cookie = Cookie::build("biscuits", "yes").finish();
    if let Ok(cookie) = HeaderValue::from_str(&cookie.to_string()) {
        response.insert_header(header::SET_COOKIE, cookie);
    }
    Ok(())
}
```

## `redirect`

One common modification to an HTTP response is to redirect to another page. The Actix and Axum integrations provide a `redirect` function to make this easy to do.

```rust
#[server]
pub async fn login(
    username: String,
    password: String,
    remember: Option<String>,
) -> Result<(), ServerFnError> {
    // pull the DB pool and auth provider from context
    let pool = pool()?;
    let auth = auth()?;

    // check whether the user exists
    let user: User = User::get_from_username(username, &pool)
        .await
        .ok_or_else(|| {
            ServerFnError::ServerError("User does not exist.".into())
        })?;

    // check whether the user has provided the correct password
    match verify(password, &user.password)? {
        // if the password is correct...
        true => {
            // log the user in
            auth.login_user(user.id);
            auth.remember_user(remember.is_some());

            // and redirect to the home page
            leptos_axum::redirect("/");
            Ok(())
        }
        // if not, return an error
        false => Err(ServerFnError::ServerError(
            "Password does not match.".to_string(),
        )),
    }
}
```

This server function can then be used from your application. This `redirect` works well with the progressively-enhanced `<ActionForm/>` component: without JS/WASM, the server response will redirect because of the status code and header. With JS/WASM, the `<ActionForm/>` will detect the redirect in the server function response, and use client-side navigation to redirect to the new page.
