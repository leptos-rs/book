# Error Handling

[In the last chapter](./06_control_flow.md), we saw that you can render `Option<T>`:
in the `None` case, it will render nothing, and in the `Some(T)` case, it will render `T`
(that is, if `T` implements `IntoView`). You can actually do something very similar
with a `Result<T, E>`. In the `Err(_)` case, it will render nothing. In the `Ok(T)`
case, it will render the `T`.

Let’s start with a simple component to capture a number input.

```rust
#[component]
fn NumericInput() -> impl IntoView {
    let (value, set_value) = signal(Ok(0));

    view! {
        <label>
            "Type an integer (or not!)"
            <input type="number" on:input:target=move |ev| {
              // when input changes, try to parse a number from the input
              set_value.set(ev.target().value().parse::<i32>())
            }/>
            <p>
                "You entered "
                <strong>{value}</strong>
            </p>
        </label>
    }
}
```

Every time you change the input, `on_input` will attempt to parse its value into a 32-bit
integer (`i32`), and store it in our `value` signal, which is a `Result<i32, _>`. If you
type the number `42`, the UI will display

```
You entered 42
```

But if you type the string `foo`, it will display

```
You entered
```

This is not great. It saves us using `.unwrap_or_default()` or something, but it would be
much nicer if we could catch the error and do something with it.

You can do that, with the [`<ErrorBoundary/>`](https://docs.rs/leptos/latest/leptos/fn.ErrorBoundary.html)
component.

```admonish note
People often try to point out that `<input type="number">` prevents you from typing a string 
like `foo`, or anything else that's not a number. This is true in some browsers, but not in all!
Moreover, there are a variety of things that can be typed into a plain number input that are not an
`i32`: a floating-point number, a larger-than-32-bit number, the letter `e`, and so on. The browser
can be told to uphold some of these invariants, but browser behavior still varies: Parsing for yourself
is important!
```

## `<ErrorBoundary/>`

An `<ErrorBoundary/>` is a little like the `<Show/>` component we saw in the last chapter.
If everything’s okay—which is to say, if everything is `Ok(_)`—it renders its children.
But if there’s an `Err(_)` rendered among those children, it will trigger the
`<ErrorBoundary/>`’s `fallback`.

Let’s add an `<ErrorBoundary/>` to this example.

```rust
#[component]
fn NumericInput() -> impl IntoView {
        let (value, set_value) = signal(Ok(0));

    view! {
        <h1>"Error Handling"</h1>
        <label>
            "Type a number (or something that's not a number!)"
            <input type="number" on:input:target=move |ev| {
                // when input changes, try to parse a number from the input
                set_value.set(ev.target().value().parse::<i32>())
            }/>
            // If an `Err(_) had been rendered inside the <ErrorBoundary/>,
            // the fallback will be displayed. Otherwise, the children of the
            // <ErrorBoundary/> will be displayed.
            <ErrorBoundary
                // the fallback receives a signal containing current errors
                fallback=|errors| view! {
                    <div class="error">
                        <p>"Not a number! Errors: "</p>
                        // we can render a list of errors
                        // as strings, if we'd like
                        <ul>
                            {move || errors.get()
                                .into_iter()
                                .map(|(_, e)| view! { <li>{e.to_string()}</li>})
                                .collect::<Vec<_>>()
                            }
                        </ul>
                    </div>
                }
            >
                <p>
                    "You entered "
                    // because `value` is `Result<i32, _>`,
                    // it will render the `i32` if it is `Ok`,
                    // and render nothing and trigger the error boundary
                    // if it is `Err`. It's a signal, so this will dynamically
                    // update when `value` changes
                    <strong>{value}</strong>
                </p>
            </ErrorBoundary>
        </label>
    }
}
```

Now, if you type `42`, `value` is `Ok(42)` and you’ll see

```
You entered 42
```

If you type `foo`, value is `Err(_)` and the `fallback` will render. We’ve chosen to render
the list of errors as a `String`, so you’ll see something like

```
Not a number! Errors:
- cannot parse integer from empty string
```

If you fix the error, the error message will disappear and the content you’re wrapping in
an `<ErrorBoundary/>` will appear again.

```admonish sandbox title="Live example" collapsible=true

[Click to open CodeSandbox.](https://codesandbox.io/p/devbox/7-errors-0-7-qqywqz?file=%2Fsrc%2Fmain.rs%3A5%2C1-46%2C6&workspaceId=478437f3-1f86-4b1e-b665-5c27a31451fb)

<noscript>
  Please enable JavaScript to view examples.
</noscript>

<template>
  <iframe src="https://codesandbox.io/p/devbox/7-errors-0-7-qqywqz?file=%2Fsrc%2Fmain.rs%3A5%2C1-46%2C6&workspaceId=478437f3-1f86-4b1e-b665-5c27a31451fb" width="100%" height="1000px" style="max-height: 100vh"></iframe>
</template>
```

<details>
<summary>CodeSandbox Source</summary>

```rust
use leptos::prelude::*;

#[component]
fn App() -> impl IntoView {
    let (value, set_value) = signal(Ok(0));

    view! {
        <h1>"Error Handling"</h1>
        <label>
            "Type a number (or something that's not a number!)"
            <input type="number" on:input:target=move |ev| {
                // when input changes, try to parse a number from the input
                set_value.set(ev.target().value().parse::<i32>())
            }/>
            // If an `Err(_) had been rendered inside the <ErrorBoundary/>,
            // the fallback will be displayed. Otherwise, the children of the
            // <ErrorBoundary/> will be displayed.
            <ErrorBoundary
                // the fallback receives a signal containing current errors
                fallback=|errors| view! {
                    <div class="error">
                        <p>"Not a number! Errors: "</p>
                        // we can render a list of errors
                        // as strings, if we'd like
                        <ul>
                            {move || errors.get()
                                .into_iter()
                                .map(|(_, e)| view! { <li>{e.to_string()}</li>})
                                .collect::<Vec<_>>()
                            }
                        </ul>
                    </div>
                }
            >
                <p>
                    "You entered "
                    // because `value` is `Result<i32, _>`,
                    // it will render the `i32` if it is `Ok`,
                    // and render nothing and trigger the error boundary
                    // if it is `Err`. It's a signal, so this will dynamically
                    // update when `value` changes
                    <strong>{value}</strong>
                </p>
            </ErrorBoundary>
        </label>
    }
}

fn main() {
    leptos::mount::mount_to_body(App)
}
```

</details>
</preview>
