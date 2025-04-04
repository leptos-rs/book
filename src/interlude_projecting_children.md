# Projecting Children

As you build components you may occasionally find yourself wanting to “project” children through multiple layers of components.

## The Problem

Consider the following:

```rust
pub fn NestedShow<F, IV>(fallback: F, children: ChildrenFn) -> impl IntoView
where
    F: Fn() -> IV + Send + Sync + 'static,
    IV: IntoView + 'static,
{
    view! {
        <Show
            when=|| todo!()
            fallback=|| ()
        >
            <Show
                when=|| todo!()
                fallback=fallback
            >
                {children()}
            </Show>
        </Show>
    }
}
```

This is pretty straightforward: if the inner condition is `true`, we want to show `children`. If not, we want to show `fallback`. And if the outer condition is `false`, we just render `()`, i.e., nothing.

In other words, we want to pass the children of `<NestedShow/>` _through_ the outer `<Show/>` component to become the children of the inner `<Show/>`. This is what I mean by “projection.”

This won’t compile.

```
error[E0525]: expected a closure that implements the `Fn` trait, but this closure only implements `FnOnce`
```

Each `<Show/>` needs to be able to construct its `children` multiple times. The first time you construct the outer `<Show/>`’s children, it takes `fallback` and `children` to move them into the invocation of the inner `<Show/>`, but then they're not available for future outer-`<Show/>` children construction.

## The Details

> Feel free to skip ahead to the solution.

If you want to really understand the issue here, it may help to look at the expanded `view` macro. Here’s a cleaned-up version:

```rust
Show(
    ShowProps::builder()
        .when(|| todo!())
        .fallback(|| ())
        .children({
            // children and fallback are moved into a closure here
            ::leptos::children::ToChildren::to_children(move || {
                Show(
                    ShowProps::builder()
                        .when(|| todo!())
                        // fallback is consumed here
                        .fallback(fallback)
                        .children({
                            // children is captured here
                            ::leptos::children::ToChildren::to_children(
                                move || children(),
                            )
                        })
                        .build(),
                )
            })
        })
        .build(),
)
```

All components own their props; so the `<Show/>` in this case can’t be called because it only has captured references to `fallback` and `children`.

## Solution

However, both `<Suspense/>` and `<Show/>` take `ChildrenFn`, i.e., their `children` should implement the `Fn` type so they can be called multiple times with only an immutable reference. This means we don’t need to own `children` or `fallback`; we just need to be able to pass `'static` references to them.

We can solve this problem by using the [`StoredValue`](https://docs.rs/leptos/latest/leptos/reactive/owner/struct.StoredValue.html) primitive. This essentially stores a value in the reactive system, handing ownership off to the framework in exchange for a reference that is, like signals, `Copy` and `'static`, which we can access or modify through certain methods.

In this case, it’s really simple:

```rust
pub fn NestedShow<F, IV>(fallback: F, children: ChildrenFn) -> impl IntoView
where
    F: Fn() -> IV + Send + Sync + 'static,
    IV: IntoView + 'static,
{
    let fallback = StoredValue::new(fallback);
    let children = StoredValue::new(children);

    view! {
        <Show
            when=|| todo!()
            fallback=|| ()
        >
            <Show
                // check whether user is verified
                // by reading from the resource
                when=move || todo!()
                fallback=move || fallback.read_value()()
            >
                {children.read_value()()}
            </Show>
        </Show>
    }
}
```

At the top level, we store both `fallback` and `children` in the reactive scope owned by `NestedShow`. Now we can simply move those references down through the other layers into the `<Show/>` component and call them there.

## A Final Note

Note that this works because `<Show/>` only needs an immutable reference to their children (which `.read_value` can give), not ownership.

In other cases, you may need to project owned props through a function that takes `ChildrenFn` and therefore needs to be called more than once. In this case, you may find the `clone:` helper in the`view` macro helpful.

Consider this example

```rust
#[component]
pub fn App() -> impl IntoView {
    let name = "Alice".to_string();
    view! {
        <Outer>
            <Inner>
                <Inmost name=name.clone()/>
            </Inner>
        </Outer>
    }
}

#[component]
pub fn Outer(children: ChildrenFn) -> impl IntoView {
    children()
}

#[component]
pub fn Inner(children: ChildrenFn) -> impl IntoView {
    children()
}

#[component]
pub fn Inmost(name: String) -> impl IntoView {
    view! {
        <p>{name}</p>
    }
}
```

Even with `name=name.clone()`, this gives the error

```
cannot move out of `name`, a captured variable in an `Fn` closure
```

It’s captured through multiple levels of children that need to run more than once, and there’s no obvious way to clone it _into_ the children.

In this case, the `clone:` syntax comes in handy. Calling `clone:name` will clone `name` _before_ moving it into `<Inner/>`’s children, which solves our ownership issue.

```rust
view! {
	<Outer>
		<Inner clone:name>
			<Inmost name=name.clone()/>
		</Inner>
	</Outer>
}
```

These issues can be a little tricky to understand or debug, because of the opacity of the `view` macro. But in general, they can always be solved.
