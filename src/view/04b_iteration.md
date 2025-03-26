# Iterating over More Complex Data with `<For/>`

This chapter goes into iteration over nested data structures in a bit
more depth. It belongs here with the other chapter on iteration, but feel
free to skip it and come back if you’d like to stick with simpler subjects
for now.

## The Problem

I just said that the framework does not rerender any of the items in one of the
rows, unless the key has changed. This probably makes sense at first, but it can
easily trip you up.

Let’s consider an example in which each of the items in our row is some data structure.
Imagine, for example, that the items come from some JSON array of keys and values:

```rust
#[derive(Debug, Clone)]
struct DatabaseEntry {
    key: String,
    value: i32,
}
```

Let’s define a simple component that will iterate over the rows and display each one:

```rust
#[component]
pub fn App() -> impl IntoView {
    // start with a set of three rows
    let (data, set_data) = signal(vec![
        DatabaseEntry {
            key: "foo".to_string(),
            value: 10,
        },
        DatabaseEntry {
            key: "bar".to_string(),
            value: 20,
        },
        DatabaseEntry {
            key: "baz".to_string(),
            value: 15,
        },
    ]);
    view! {
        // when we click, update each row,
        // doubling its value
        <button on:click=move |_| {
            set_data.update(|data| {
                for row in data {
                    row.value *= 2;
                }
            });
            // log the new value of the signal
            leptos::logging::log!("{:?}", data.get());
        }>
            "Update Values"
        </button>
        // iterate over the rows and display each value
        <For
            each=move || data.get()
            key=|state| state.key.clone()
            let(child)
        >
            <p>{child.value}</p>
        </For>
    }
}
```

> Note the `let(child)` syntax here. In the previous chapter we introduced `<For/>`
> with a `children` prop. We can actually create this value directly in the children
> of the `<For/>` component, without breaking out of the `view` macro: the `let:child`
> combined with `<p>{child.value}</p>` above is the equivalent of
>
> ```rust
> children=|child| view! { <p>{child.value}</p> }
> ```
>
> For convenience, you can also choose to destructure the pattern of your data:
>
> ```rust
> <For
>     each=move || data.get()
>     key=|state| state.key.clone()
>     let(DatabaseEntry { key, value })
> >
> ```

When you click the `Update Values` button... nothing happens. Or rather:
the signal is updated, the new value is logged, but the `{child.value}`
for each row doesn’t update.

Let’s see: is that because we forgot to add a closure to make it reactive?
Let’s try `{move || child.value}`.

...Nope. Still nothing.

Here’s the problem: as I said, each row is only rerendered when the key changes.
We’ve updated the value for each row, but not the key for any of the rows, so
nothing has rerendered. And if you look at the type of `child.value`, it’s a plain
`i32`, not a reactive `ReadSignal<i32>` or something. This means that even if we
wrap a closure around it, the value in this row will never update.

We have three possible solutions:

1. change the `key` so that it always updates when the data structure changes
2. change the `value` so that it’s reactive
3. take a reactive slice of the data structure instead of using each row directly

## Option 1: Change the Key

Each row is only rerendered when the key changes. Our rows above didn’t rerender,
because the key didn’t change. So: why not just force the key to change?

```rust
<For
	each=move || data.get()
	key=|state| (state.key.clone(), state.value)
	let(child)
>
	<p>{child.value}</p>
</For>
```

Now we include both the key and the value in the `key`. This means that whenever the
value of a row changes, `<For/>` will treat it as if it’s an entirely new row, and
replace the previous one.

### Pros

This is very easy. We can make it even easier by deriving `PartialEq`, `Eq`, and `Hash`
on `DatabaseEntry`, in which case we could just `key=|state| state.clone()`.

### Cons

**This is the least efficient of the three options.** Every time the value of a row
changes, it throws out the previous `<p>` element and replaces it with an entirely new
one. Rather than making a fine-grained update to the text node, in other words, it really
does rerender the entire row on every change, and this is expensive in proportion to how
complex the UI of the row is.

You’ll notice we also end up cloning the whole data structure so that `<For/>` can hold
onto a copy of the key. For more complex structures, this can become a bad idea fast!

## Option 2: Nested Signals

If we do want that fine-grained reactivity for the value, one option is to wrap the `value`
of each row in a signal.

```rust
#[derive(Debug, Clone)]
struct DatabaseEntry {
    key: String,
    value: RwSignal<i32>,
}
```

`RwSignal<_>` is a “read-write signal,” which combines the getter and setter in one object.
I’m using it here because it’s a little easier to store in a struct than separate getters
and setters.

```rust
#[component]
pub fn App() -> impl IntoView {
    // start with a set of three rows
    let (data, set_data) = signal(vec![
        DatabaseEntry {
            key: "foo".to_string(),
            value: RwSignal::new(10),
        },
        DatabaseEntry {
            key: "bar".to_string(),
            value: RwSignal::new(20),
        },
        DatabaseEntry {
            key: "baz".to_string(),
            value: RwSignal::new(15),
        },
    ]);
    view! {
        // when we click, update each row,
        // doubling its value
        <button on:click=move |_| {
            for row in &*data.read() {
                row.value.update(|value| *value *= 2);
            }
            // log the new value of the signal
            leptos::logging::log!("{:?}", data.get());
        }>
            "Update Values"
        </button>
        // iterate over the rows and display each value
        <For
            each=move || data.get()
            key=|state| state.key.clone()
            let(child)
        >
            <p>{child.value}</p>
        </For>
    }
}
```

This version works! And if you look in the DOM inspector in your browser, you’ll
see that unlike in the previous version, in this version only the individual text
nodes are updated. Passing the signal directly into `{child.value}` works, as
signals do keep their reactivity if you pass them into the view.

Note that I changed the `set_data.update()` to a `data.read()`. `.read()` is a
non-cloning way of accessing a signal’s value. In this case, we are only updating
the inner values, not updating the list of values: because signals maintain their
own state, we don’t actually need to update the `data` signal at all, so the immutable
`.read()` is fine here.

> In fact, this version doesn’t update `data`, so the `<For/>` is essentially a static
> list as in the last chapter, and this could just be a plain iterator. But the `<For/>`
> is useful if we want to add or remove rows in the future.

### Pros

This is the most efficient option, and fits directly with the rest of the mental model
of the framework: values that change over time are wrapped in signals so the interface
can respond to them.

### Cons

Nested reactivity can be cumbersome if you’re receiving data from an API or another
data source you don’t control, and you don’t want to create a different struct wrapping
each field in a signal.

## Option 3: Memoized Slices

Leptos provides a primitive called a [`Memo`](https://docs.rs/leptos/latest/leptos/reactive/computed/struct.Memo.html),
which creates a derived computation that only triggers a reactive update when its value
has changed.

This allows you to create reactive values for subfields of a larger data structure,
without needing to wrap the fields of that structure in signals.

Most of the application can remain the same as the initial (broken) version, but the `<For/>`
will be updated to this:

```rust
<For
    each=move || data.get().into_iter().enumerate()
    key=|(_, state)| state.key.clone()
    children=move |(index, _)| {
        let value = Memo::new(move |_| {
            data.with(|data| data.get(index).map(|d| d.value).unwrap_or(0))
        });
        view! {
            <p>{value}</p>
        }
    }
/>
```

You’ll notice a few differences here:

- we convert the `data` signal into an enumerated iterator
- we use the `children` prop explicitly, to make it easier to run some non-`view` code
- we define a `value` memo and use that in the view. This `value` field doesn’t actually
  use the `child` being passed into each row. Instead, it uses the index and reaches back
  into the original `data` to get the value.

Every time `data` changes, now, each memo will be recalculated. If its value has changed,
it will update its text node, without rerendering the whole row.

### Pros

We get the same fine-grained reactivity of the signal-wrapped version, without needing to
wrap the data in signals.

### Cons

It’s a bit more complex to set up this memo-per-row inside the `<For/>` loop rather than
using nested signals. For example, you’ll notice that we have to guard against the possibility
that the `data[index]` would panic by using `data.get(index)`, because this memo may be
triggered to re-run once just after the row is removed. (This is because the memo for each row
and the whole `<For/>` both depend on the same `data` signal, and the order of execution for
multiple reactive values that depend on the same signal isn’t guaranteed.)

Note also that while memos memoize their reactive changes, the same
calculation does need to re-run to check the value every time, so nested reactive signals
will still be more efficient for pinpoint updates here.

## Option 4: Stores

> Some of this content is duplicated in the section on global state management with stores [here](../15_global_state.md#option-3-create-a-global-state-store). Both sections are intermediate/optional content, so I thought some duplication couldn’t hurt.

Leptos 0.7 introduces a new reactive primitive called “stores.” Stores are designed to address
the issues described in this chapter so far. They’re a bit experimental, so they require an additional dependency called `reactive_stores` in your `Cargo.toml`.

Stores give you fine-grained reactive access to the individual fields of a struct, and to individual items in collections like `Vec<_>`, without needing to create nested signals or memos manually, as in the options given above.

Stores are built on top of the `Store` derive macro, which creates a getter for each field of a struct. Calling this getter gives reactive access to that particular field. Reading from it will track only that field and its parents/children, and updating it will only notify that field and its parents/children, but not siblings. In other words, mutating `value` will not notify `key`, and so on.

We can adapt the data types we used in the examples above.

The top level of a store always needs to be a struct, so we’ll create a `Data` wrapper with a single `rows` field.
```rust
#[derive(Store, Debug, Clone)]
pub struct Data {
    #[store(key: String = |row| row.key.clone())]
    rows: Vec<DatabaseEntry>,
}

#[derive(Store, Debug, Clone)]
struct DatabaseEntry {
    key: String,
    value: i32,
}
```
Adding `#[store(key)]` to the `rows` field allows us to have keyed access to the fields of the store, which will be useful in the `<For/>` component below. We can simply use `key`, the same key that we’ll use in `<For/>`.

The `<For/>` component is pretty straightforward:
```rust
<For
    each=move || data.rows()
    key=|row| row.read().key.clone()
    children=|child| {
        let value = child.value();
        view! { <p>{move || value.get()}</p> }
    }
/>
```
Because `rows` is a keyed field, it implements `IntoIterator`, and we can simply use `move || data.rows()` as the `each` prop. This will react to any changes to the `rows` list, just as `move || data.get()` did in our nested-signal version.

The `key` field calls `.read()` to get access to the current value of the row, then clones and returns the `key` field.

In `children` prop, calling `child.value()` gives us reactive access to the `value` field for the row with this key. If rows are reordered, added, or removed, the keyed store field will keep in sync so that this `value` is always associated with the correct key.

In the update button handler, we’ll iterate over the entries in `rows`, updating each one:
```rust
for row in data.rows().iter_unkeyed() {
    *row.value().write() *= 2;
}
```

### Pros

We get the fine-grained reactivity of the nested-signal and memo versions, without needing to manually create nested signals or memoized slices. We can work with plain data (a struct and `Vec<_>`), annotated with a derive macro, rather than special nested reactive types.

Personally, I think the stores version is the nicest one here. And no surprise, as it’s the newest API. We’ve had a few years to think about these things and stores include some of the lessons we’ve learned!

### Cons

On the other hand, it’s the newest API. As of writing this sentence (December 2024), stores have only been released for a few weeks; I am sure that there are still some bugs or edge cases to be figured out.


### Full Example

Here’s the complete store example. You can find another, more complete example [here](https://github.com/leptos-rs/leptos/blob/main/examples/stores/src/lib.rs), and more discussion in the book [here](../15_global_state.md).
```
#[component]
pub fn App() -> impl IntoView {
    // instead of a signal with the rows, we create a store for Data
    let data = Store::new(Data {
        rows: vec![
            DatabaseEntry {
                key: "foo".to_string(),
                value: 10,
            },
            DatabaseEntry {
                key: "bar".to_string(),
                value: 20,
            },
            DatabaseEntry {
                key: "baz".to_string(),
                value: 15,
            },
        ],
    });

    view! {
        // when we click, update each row,
        // doubling its value
        <button on:click=move |_| {
            // allows iterating over the entries in an iterable store field
            use reactive_stores::StoreFieldIterator;

            // calling rows() gives us access to the rows 
            for row in data.rows().iter_unkeyed() {
                *row.value().write() *= 2;
            }
            // log the new value of the signal
            leptos::logging::log!("{:?}", data.get());
        }>
            "Update Values"
        </button>
        // iterate over the rows and display each value
        <For
            each=move || data.rows()
            key=|row| row.read().key.clone()
            children=|child| {
                let value = child.value();
                view! { <p>{move || value.get()}</p> }
            }
        />
    }
}
```
