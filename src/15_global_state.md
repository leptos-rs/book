# Global State Management

So far, we've only been working with local state in components, and we’ve seen how to coordinate state between parent and child components. On occasion, there are times where people look for a more general solution for global state management that can work throughout an application.

In general, **you do not need this chapter.** The typical pattern is to compose your application out of components, each of which manages its own local state, not to store all state in a global structure. However, there are some cases (like theming, saving user settings, or sharing data between components in different parts of your UI) in which you may want to use some kind of global state management.

The three best approaches to global state are

1. Using the router to drive global state via the URL
2. Passing signals through context
3. Creating a global state struct using stores

## Option #1: URL as Global State

In many ways, the URL is actually the best way to store global state. It can be accessed from any component, anywhere in your tree. There are native HTML elements like `<form>` and `<a>` that exist solely to update the URL. And it persists across page reloads and between devices; you can share a URL with a friend or send it from your phone to your laptop and any state stored in it will be replicated.

The next few sections of the tutorial will be about the router, and we’ll get much more into these topics.

But for now, we'll just look at options #2 and #3.

## Option #2: Passing Signals through Context

In the section on [parent-child communication](view/08_parent_child.md), we saw that you can use `provide_context` to pass signal from a parent component to a child, and `use_context` to read it in the child. But `provide_context` works across any distance. If you want to create a global signal that holds some piece of state, you can provide it and access it via context anywhere in the descendants of the component where you provide it.

A signal provided via context only causes reactive updates where it is read, not in any of the components in between, so it maintains the power of fine-grained reactive updates, even at a distance.

We start by creating a signal in the root of the app and providing it to
all its children and descendants using `provide_context`.

```rust
#[component]
fn App() -> impl IntoView {
    // here we create a signal in the root that can be consumed
    // anywhere in the app.
    let (count, set_count) = signal(0);
    // we'll pass the setter to specific components,
    // but provide the count itself to the whole app via context
    provide_context(count);

    view! {
        // SetterButton is allowed to modify the count
        <SetterButton set_count/>
        // These consumers can only read from it
        // But we could give them write access by passing `set_count` if we wanted
        <FancyMath/>
        <ListItems/>
    }
}
```

`<SetterButton/>` is the kind of counter we’ve written several times now.

`<FancyMath/>` and `<ListItems/>` both consume the signal we’re providing via
`use_context` and do something with it.

```rust
/// A component that does some "fancy" math with the global count
#[component]
fn FancyMath() -> impl IntoView {
    // here we consume the global count signal with `use_context`
    let count = use_context::<ReadSignal<u32>>()
        // we know we just provided this in the parent component
        .expect("there to be a `count` signal provided");
    let is_even = move || count.get() & 1 == 0;

    view! {
        <div class="consumer blue">
            "The number "
            <strong>{count}</strong>
            {move || if is_even() {
                " is"
            } else {
                " is not"
            }}
            " even."
        </div>
    }
}
```

## Option #3: Create a Global State Store

> Some of this content is duplicated from the section on complex iteration with stores [here](../view/04b_iteration.md#option-4-stores). Both sections are intermediate/optional content, so I thought some duplication couldn’t hurt.

Stores are a new reactive primitive, available in Leptos 0.7 through the accompanying `reactive_stores` crate. (This crate is shipped separately for now so we can continue to develop it without requiring a version change to the whole framework.)

Stores allow you to wrap an entire struct, and reactively read from and update individual fields without tracking changes to other fields.

They are used by adding `#[derive(Store)]` onto a struct. (You can `use reactive_stores::Store;` to import the macro.) This creates an extension trait with a getter for each field of the struct, when the struct is wrapped in a `Store<_>`.

```rust
#[derive(Clone, Debug, Default, Store)]
struct GlobalState {
    count: i32,
    name: String,
}
```

This creates a trait named `GlobalStateStoreFields` which adds with methods `count` and `name` to a `Store<GlobalState>`. Each method returns a reactive store *field*.

```rust
#[component]
fn App() -> impl IntoView {
    provide_context(Store::new(GlobalState::default()));

    // etc.
}

/// A component that updates the count in the global state.
#[component]
fn GlobalStateCounter() -> impl IntoView {
    let state = expect_context::<Store<GlobalState>>();

    // this gives us reactive access to the `count` field only
    let count = state.count();

    view! {
        <div class="consumer blue">
            <button
                on:click=move |_| {
                    *count.write() += 1;
                }
            >
                "Increment Global Count"
            </button>
            <br/>
            <span>"Count is: " {move || count.get()}</span>
        </div>
    }
}
```

Clicking this button only updates `state.count`. If we read from `state.name` somewhere else, 
click the button won’t notify it. This allows you to combine the benefits of a top-down
data flow and of fine-grained reactive updates.

Check out the [`stores` example](https://github.com/leptos-rs/leptos/blob/main/examples/stores/src/lib.rs) in the repo for a more extensive example.
