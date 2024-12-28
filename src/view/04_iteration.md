# Iteration

Whether you’re listing todos, displaying a table, or showing product images,
iterating over a list of items is a common task in web applications. Reconciling
the differences between changing sets of items can also be one of the trickiest
tasks for a framework to handle well.

Leptos supports two different patterns for iterating over items:

1. For static views: `Vec<_>`
2. For dynamic lists: `<For/>`

## Static Views with `Vec<_>`

Sometimes you need to show an item repeatedly, but the list you’re drawing from
does not often change. In this case, it’s important to know that you can insert
any `Vec<IV> where IV: IntoView` into your view. In other words, if you can render
`T`, you can render `Vec<T>`.

```rust
let values = vec![0, 1, 2];
view! {
    // this will just render "012"
    <p>{values.clone()}</p>
    // or we can wrap them in <li>
    <ul>
        {values.into_iter()
            .map(|n| view! { <li>{n}</li>})
            .collect::<Vec<_>>()}
    </ul>
}
```

Leptos also provides a `.collect_view()` helper function that allows you to collect any iterator of `T: IntoView` into `Vec<View>`.

```rust
let values = vec![0, 1, 2];
view! {
    // this will just render "012"
    <p>{values.clone()}</p>
    // or we can wrap them in <li>
    <ul>
        {values.into_iter()
            .map(|n| view! { <li>{n}</li>})
            .collect_view()}
    </ul>
}
```

The fact that the _list_ is static doesn’t mean the interface needs to be static.
You can render dynamic items as part of a static list.

```rust
// create a list of 5 signals
let length = 5;
let counters = (1..=length).map(|idx| RwSignal::new(idx));
```

Note here that instead of calling `signal()` to get a tuple with a reader and a writer,
here we use `RwSignal::new()` to get a single, read-write signal. This is just more convenient
for a situation where we’d otherwise be passing the tuples around.

```
// each item manages a reactive view
// but the list itself will never change
let counter_buttons = counters
    .map(|(count, set_count)| {
        view! {
            <li>
                <button
                    on:click=move |_| *set_count.write() += 1
                >
                    {count}
                </button>
            </li>
        }
    })
    .collect_view();

view! {
    <ul>{counter_buttons}</ul>
}
```

You _can_ render a `Fn() -> Vec<_>` reactively as well. But note that this is an unkeyed
list update: it will reuse the existing DOM elements, and update them with the new values,
according to their order in the new `Vec<_>`. If you’re just adding and removing items at the 
end of the list, this works well, but if you are moving items around or inserting items into 
the middle of the list, this will cause the browser to do more work than it needs to, and may 
have surprising effects on things like input state and CSS animations. (For more on the “keyed”
vs. “unkeyed” distinction, and some practical examples, you can read
[this article](https://www.stefankrause.net/wp/?p=342).)

Luckily, there’s an efficient way to do keyed list iteration, as well.

## Dynamic Rendering with the `<For/>` Component

The [`<For/>`](https://docs.rs/leptos/latest/leptos/control_flow/fn.For.html) component is a
keyed dynamic list. It takes three props:

- `each`: a reactive function that returns the items `T` to be iterated over
- `key`: a key function that takes `&T` and returns a stable, unique key or ID
- `children`: renders each `T` into a view

`key` is, well, the key. You can add, remove, and move items within the list. As
long as each item’s key is stable over time, the framework does not need to rerender
any of the items, unless they are new additions, and it can very efficiently add,
remove, and move items as they change. This allows for extremely efficient updates
to the list as it changes, with minimal additional work.

Creating a good `key` can be a little tricky. You generally do _not_ want to use
an index for this purpose, as it is not stable—if you remove or move items, their
indices change.

But it’s a great idea to do something like generating a unique ID for each row as
it is generated, and using that as an ID for the key function.

Check out the `<DynamicList/>` component below for an example.

```admonish sandbox title="Live example" collapsible=true

[Click to open CodeSandbox.](https://codesandbox.io/p/devbox/4-iteration-0-7-dw4dfl?file=%2Fsrc%2Fmain.rs%3A1%2C1-159%2C1&workspaceId=478437f3-1f86-4b1e-b665-5c27a31451fb)

<noscript>
  Please enable JavaScript to view examples.
</noscript>

<template>
  <iframe src="https://codesandbox.io/p/devbox/4-iteration-0-7-dw4dfl?file=%2Fsrc%2Fmain.rs%3A1%2C1-159%2C1&workspaceId=478437f3-1f86-4b1e-b665-5c27a31451fb" width="100%" height="1000px" style="max-height: 100vh"></iframe>
</template>

```

<details>
<summary>CodeSandbox Source</summary>

```rust
use leptos::prelude::*;

// Iteration is a very common task in most applications.
// So how do you take a list of data and render it in the DOM?
// This example will show you the two ways:
// 1) for mostly-static lists, using Rust iterators
// 2) for lists that grow, shrink, or move items, using <For/>

#[component]
fn App() -> impl IntoView {
    view! {
        <h1>"Iteration"</h1>
        <h2>"Static List"</h2>
        <p>"Use this pattern if the list itself is static."</p>
        <StaticList length=5/>
        <h2>"Dynamic List"</h2>
        <p>"Use this pattern if the rows in your list will change."</p>
        <DynamicList initial_length=5/>
    }
}

/// A list of counters, without the ability
/// to add or remove any.
#[component]
fn StaticList(
    /// How many counters to include in this list.
    length: usize,
) -> impl IntoView {
    // create counter signals that start at incrementing numbers
    let counters = (1..=length).map(|idx| RwSignal::new(idx));

    // when you have a list that doesn't change, you can
    // manipulate it using ordinary Rust iterators
    // and collect it into a Vec<_> to insert it into the DOM
    let counter_buttons = counters
        .map(|count| {
            view! {
                <li>
                    <button
                        on:click=move |_| *count.write() += 1
                    >
                        {count}
                    </button>
                </li>
            }
        })
        .collect::<Vec<_>>();

    // Note that if `counter_buttons` were a reactive list
    // and its value changed, this would be very inefficient:
    // it would rerender every row every time the list changed.
    view! {
        <ul>{counter_buttons}</ul>
    }
}

/// A list of counters that allows you to add or
/// remove counters.
#[component]
fn DynamicList(
    /// The number of counters to begin with.
    initial_length: usize,
) -> impl IntoView {
    // This dynamic list will use the <For/> component.
    // <For/> is a keyed list. This means that each row
    // has a defined key. If the key does not change, the row
    // will not be re-rendered. When the list changes, only
    // the minimum number of changes will be made to the DOM.

    // `next_counter_id` will let us generate unique IDs
    // we do this by simply incrementing the ID by one
    // each time we create a counter
    let mut next_counter_id = initial_length;

    // we generate an initial list as in <StaticList/>
    // but this time we include the ID along with the signal
    // see NOTE in add_counter below re: ArcRwSignal
    let initial_counters = (0..initial_length)
        .map(|id| (id, ArcRwSignal::new(id + 1)))
        .collect::<Vec<_>>();

    // now we store that initial list in a signal
    // this way, we'll be able to modify the list over time,
    // adding and removing counters, and it will change reactively
    let (counters, set_counters) = signal(initial_counters);

    let add_counter = move |_| {
        // create a signal for the new counter
        // we use ArcRwSignal here, instead of RwSignal
        // ArcRwSignal is a reference-counted type, rather than the arena-allocated
        // signal types we've been using so far.
        // When we're creating a collection of signals like this, using ArcRwSignal
        // allows each signal to be deallocated when its row is removed.
        let sig = ArcRwSignal::new(next_counter_id + 1);
        // add this counter to the list of counters
        set_counters.update(move |counters| {
            // since `.update()` gives us `&mut T`
            // we can just use normal Vec methods like `push`
            counters.push((next_counter_id, sig))
        });
        // increment the ID so it's always unique
        next_counter_id += 1;
    };

    view! {
        <div>
            <button on:click=add_counter>
                "Add Counter"
            </button>
            <ul>
                // The <For/> component is central here
                // This allows for efficient, key list rendering
                <For
                    // `each` takes any function that returns an iterator
                    // this should usually be a signal or derived signal
                    // if it's not reactive, just render a Vec<_> instead of <For/>
                    each=move || counters.get()
                    // the key should be unique and stable for each row
                    // using an index is usually a bad idea, unless your list
                    // can only grow, because moving items around inside the list
                    // means their indices will change and they will all rerender
                    key=|counter| counter.0
                    // `children` receives each item from your `each` iterator
                    // and returns a view
                    children=move |(id, count)| {
                        // we can convert our ArcRwSignal to a Copy-able RwSignal
                        // for nicer DX when moving it into the view
                        let count = RwSignal::from(count);
                        view! {
                            <li>
                                <button
                                    on:click=move |_| *count.write() += 1
                                >
                                    {count}
                                </button>
                                <button
                                    on:click=move |_| {
                                        set_counters
                                            .write()
                                            .retain(|(counter_id, _)| {
                                                counter_id != &id
                                            });
                                    }
                                >
                                    "Remove"
                                </button>
                            </li>
                        }
                    }
                />
            </ul>
        </div>
    }
}

fn main() {
    leptos::mount::mount_to_body(App)
}
```

</details>
</preview>
