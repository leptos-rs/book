# Components and Props

So far, we’ve been building our whole application in a single component. This
is fine for really tiny examples, but in any real application you’ll need to
break the user interface out into multiple components, so you can break your
interface down into smaller, reusable, composable chunks.

Let’s take our progress bar example. Imagine that you want two progress bars
instead of one: one that advances one tick per click, one that advances two ticks
per click.

You _could_ do this by just creating two `<progress>` elements:

```rust
let (count, set_count) = signal(0);
let double_count = move || count.get() * 2;

view! {
    <progress
        max="50"
        value=count
    />
    <progress
        max="50"
        value=double_count
    />
}
```

But of course, this doesn’t scale very well. If you want to add a third progress
bar, you need to add this code another time. And if you want to edit anything
about it, you need to edit it in triplicate.

Instead, let’s create a `<ProgressBar/>` component.

```rust
#[component]
fn ProgressBar() -> impl IntoView {
    view! {
        <progress
            max="50"
            // hmm... where will we get this from?
            value=progress
        />
    }
}
```

There’s just one problem: `progress` is not defined. Where should it come from?
When we were defining everything manually, we just used the local variable names.
Now we need some way to pass an argument into the component.

## Component Props

We do this using component properties, or “props.” If you’ve used another frontend
framework, this is probably a familiar idea. Basically, properties are to components
as attributes are to HTML elements: they let you pass additional information into
the component.

In Leptos, you define props by giving additional arguments to the component function.

```rust
#[component]
fn ProgressBar(
    progress: ReadSignal<i32>
) -> impl IntoView {
    view! {
        <progress
            max="50"
            // now this works
            value=progress
        />
    }
}
```

Now we can use our component in the main `<App/>` component’s view.

```rust
#[component]
fn App() -> impl IntoView {
    let (count, set_count) = signal(0);
    view! {
        <button on:click=move |_| *set_count.write() += 1>
            "Click me"
        </button>
        // now we use our component!
        <ProgressBar progress=count/>
    }
}
```

Using a component in the view looks a lot like using an HTML element. You’ll
notice that you can easily tell the difference between an element and a component
because components always have `PascalCase` names. You pass the `progress` prop
in as if it were an HTML element attribute. Simple.

### Reactive and Static Props

You’ll notice that throughout this example, `progress` takes a reactive
`ReadSignal<i32>`, and not a plain `i32`. This is **very important**.

Component props have no special meaning attached to them. A component is simply
a function that runs once to set up the user interface. The only way to tell the
interface to respond to changes is to pass it a signal type. So if you have a
component property that will change over time, like our `progress`, it should
be a signal.

### `optional` Props

Right now the `max` setting is hard-coded. Let’s take that as a prop too. But
let’s make this prop optional. We can do that by annotating it with `#[prop(optional)]`.

```rust
#[component]
fn ProgressBar(
    // mark this prop optional
    // you can specify it or not when you use <ProgressBar/>
    #[prop(optional)]
    max: u16,
    progress: ReadSignal<i32>
) -> impl IntoView {
    view! {
        <progress
            max=max
            value=progress
        />
    }
}
```

Now, we can use `<ProgressBar max=50 progress=count/>`, or we can omit `max`
to use the default value (i.e., `<ProgressBar progress=count/>`). The default value
on an `optional` is its `Default::default()` value, which for a `u16` is going to
be `0`. In the case of a progress bar, a max value of `0` is not very useful.

So let’s give it a particular default value instead.

### `default` props

You can specify a default value other than `Default::default()` pretty simply
with `#[prop(default = ...)`.

```rust
#[component]
fn ProgressBar(
    #[prop(default = 100)]
    max: u16,
    progress: ReadSignal<i32>
) -> impl IntoView {
    view! {
        <progress
            max=max
            value=progress
        />
    }
}
```

### Generic Props

This is great. But we began with two counters, one driven by `count`, and one by
the derived signal `double_count`. Let’s recreate that by using `double_count`
as the `progress` prop on another `<ProgressBar/>`.

```rust,compile_fail
#[component]
fn App() -> impl IntoView {
    let (count, set_count) = signal(0);
    let double_count = move || count.get() * 2;

    view! {
        <button on:click=move |_| { set_count.update(|n| *n += 1); }>
            "Click me"
        </button>
        <ProgressBar progress=count/>
        // add a second progress bar
        <ProgressBar progress=double_count/>
    }
}
```

Hm... this won’t compile. It should be pretty easy to understand why: we’ve declared
that the `progress` prop takes `ReadSignal<i32>`, and `double_count` is not
`ReadSignal<i32>`. As rust-analyzer will tell you, its type is `|| -> i32`, i.e.,
it’s a closure that returns an `i32`.

There are a couple ways to handle this. One would be to say: “Well, I know that
for the view to be reactive, it needs to take a function or a signal. I can always
turn a signal into a function by wrapping it in a closure... Maybe I could
just take any function?” 

If you’re using nightly Rust with the `nightly` feature, signals are functions,
so you could use a generic component and take any `Fn() -> i32`:

```rust
#[component]
fn ProgressBar(
    #[prop(default = 100)]
    max: u16,
    progress: impl Fn() -> i32 + Send + Sync + 'static
) -> impl IntoView {
    view! {
        <progress
            max=max
            value=progress
        />
        // Add a line-break to avoid overlap
        <br/>
    }
}
```

> Generic props can also be specified using a `where` clause, or using inline generics like `ProgressBar<F: Fn() -> i32 + 'static>`.

Generics need to be used somewhere in the component props. This is because props are built into a struct, so all generic types must be used somewhere in the struct. This is often easily accomplished using an optional `PhantomData` prop. You can then specify a generic in the view using the syntax for expressing types: `<Component<T>/>` (not with the turbofish-style `<Component::<T>/>`).

```rust
#[component]
fn SizeOf<T: Sized>(#[prop(optional)] _ty: PhantomData<T>) -> impl IntoView {
    std::mem::size_of::<T>()
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <SizeOf<usize>/>
        <SizeOf<String>/>
    }
}
```

> Note that there are some limitations. For example, our view macro parser can’t handle nested generics like `<SizeOf<Vec<T>>/>`.

### `into` Props

If you’re on stable Rust, signals don’t directly implement `Fn()`. We could wrap the signal in a closure (`move || progress.get()`)
but that’s a bit messy.

There’s another way we could implement this, and it would be to use `#[prop(into)]`.
This attribute automatically calls `.into()` on the values you pass as props,
which allows you to easily pass props with different values.

In this case, it’s helpful to know about the
[`Signal`](https://docs.rs/leptos/latest/leptos/reactive/wrappers/read/struct.Signal.html) type. `Signal`
is an enumerated type that represents any kind of readable reactive signal, or a plain value.
It can be useful when defining APIs for components you’ll want to reuse while passing
different sorts of signals.

```rust
#[component]
fn ProgressBar(
    #[prop(default = 100)]
    max: u16,
    #[prop(into)]
    progress: Signal<i32>
) -> impl IntoView
{
    view! {
        <progress
            max=max
            value=progress
        />
        <br/>
    }
}

#[component]
fn App() -> impl IntoView {
    let (count, set_count) = signal(0);
    let double_count = move || count.get() * 2;

    view! {
        <button on:click=move |_| *set_count.write() += 1>
            "Click me"
        </button>
        // .into() converts `ReadSignal` to `Signal`
        <ProgressBar progress=count/>
        // use `Signal::derive()` to wrap a derived signal with the `Signal` type
        <ProgressBar progress=Signal::derive(double_count)/>
    }
}
```

### Optional Generic Props

Note that you can’t specify optional generic props for a component. Let’s see what would happen if you try:

```rust,compile_fail
#[component]
fn ProgressBar<F: Fn() -> i32 + Send + Sync + 'static>(
    #[prop(optional)] progress: Option<F>,
) -> impl IntoView {
    progress.map(|progress| {
        view! {
            <progress
                max=100
                value=progress
            />
            <br/>
        }
    })
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <ProgressBar/>
    }
}
```

Rust helpfully gives the error

```
xx |         <ProgressBar/>
   |          ^^^^^^^^^^^ cannot infer type of the type parameter `F` declared on the function `ProgressBar`
   |
help: consider specifying the generic argument
   |
xx |         <ProgressBar::<F>/>
   |                     +++++
```

You can specify generics on components with a `<ProgressBar<F>/>` syntax (no turbofish in the `view` macro). Specifying the correct type here is not possible; closures and functions in general are unnameable types. The compiler can display them with a shorthand, but you can’t specify them.

However, you can get around this by providing a concrete type using `Box<dyn _>` or `&dyn _`:

```rust
#[component]
fn ProgressBar(
    #[prop(optional)] progress: Option<Box<dyn Fn() -> i32 + Send + Sync>>,
) -> impl IntoView {
    progress.map(|progress| {
        view! {
            <progress
                max=100
                value=progress
            />
            <br/>
        }
    })
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <ProgressBar/>
    }
}
```

Because the Rust compiler now knows the concrete type of the prop, and therefore its size in memory even in the `None` case, this compiles fine.

> In this particular case, `&dyn Fn() -> i32` will cause lifetime issues, but in other cases, it may be a possibility.

## Documenting Components

This is one of the least essential but most important sections of this book.
It’s not strictly necessary to document your components and their props. It may
be very important, depending on the size of your team and your app. But it’s very
easy, and bears immediate fruit.

To document a component and its props, you can simply add doc comments on the
component function, and each one of the props:

```rust
/// Shows progress toward a goal.
#[component]
fn ProgressBar(
    /// The maximum value of the progress bar.
    #[prop(default = 100)]
    max: u16,
    /// How much progress should be displayed.
    #[prop(into)]
    progress: Signal<i32>,
) -> impl IntoView {
    /* ... */
}
```

That’s all you need to do. These behave like ordinary Rust doc comments, except
that you can document individual component props, which can’t be done with Rust
function arguments.

This will automatically generate documentation for your component, its `Props`
type, and each of the fields used to add props. It can be a little hard to
understand how powerful this is until you hover over the component name or props
and see the power of the `#[component]` macro combined with rust-analyzer here.

## Spreading Attributes onto Components

Sometimes you want users to be able to add additional attributes to a component. For example, you might want users to be able to add their own `class` or `id` attributes for styling or other purposes.

You _could_ do this by creating `class` or `id` props that you then apply to the appropriate element. But Leptos also supports “spreading” additional attributes onto components. Attributes added to a component will be applied to all top-level HTML elements returned from its view.

```rust
// you can create attribute lists by using the view macro with a spread {..} as the tag name
let spread_onto_component = view! {
    <{..} aria-label="a component with attribute spreading"/>
};


view! {
    // attributes that are spread onto a component will be applied to *all* elements returned as part of
    // the component's view. to apply attributes to a subset of the component, pass them via a component prop
    <ComponentThatTakesSpread
        // plain identifiers are for props
        some_prop="foo"
        another_prop=42

        // the class:, style:, prop:, on: syntaxes work just as they do on elements
        class:foo=true
        style:font-weight="bold"
        prop:cool=42
        on:click=move |_| alert("clicked ComponentThatTakesSpread")

        // to pass a plain HTML attribute, prefix it with attr:
        attr:id="foo"

        // or, if you want to include multiple attributes, rather than prefixing each with
        // attr:, you can separate them from component props with the spread {..}
        {..} // everything after this is treated as an HTML attribute
        title="ooh, a title!"

        // we can add the whole list of attributes defined above
        {..spread_onto_component}
    />
}
```

``````admonish note
If you would like to extract the attributes into the function so you can use it in multiple components, you can do so by implementing a function that returns `impl Attribute`.

That would make example above look like this:

```rust
fn spread_onto_component() -> impl Attribute {
    view!{
        <{..} aria-label="a component with attribute spreading">
    }
}

#[component]
fn UseComponentThatTakesSpread() -> impl IntoView {
    view! {
        // attributes that are spread onto a component will be applied to *all* elements returned as part of
        // the component's view. to apply attributes to a subset of the component, pass them via a component prop
        <ComponentThatTakesSpread
            // plain identifiers are for props
            some_prop="foo"
            another_prop=42

            // the class:, style:, prop:, on: syntaxes work just as they do on elements
            class:foo=true
            style:font-weight="bold"
            prop:cool=42
            on:click=move |_| alert("clicked ComponentThatTakesSpread")

            // to pass a plain HTML attribute, prefix it with attr:
            attr:id="foo"

            // or, if you want to include multiple attributes, rather than prefixing each with
            // attr:, you can separate them from component props with the spread {..}
            {..} // everything after this is treated as an HTML attribute
            title="ooh, a title!"

            // we can add the whole list of attributes defined above
            {..spread_onto_component()}
        />
    }
}
```
``````

See the [`spread` example](https://github.com/leptos-rs/leptos/blob/main/examples/spread/src/lib.rs) for more examples.

```admonish sandbox title="Live example" collapsible=true

[Click to open CodeSandbox.](https://codesandbox.io/p/devbox/3-components-0-7-rkjn3j?file=%2Fsrc%2Fmain.rs%3A39%2C10)

<noscript>
  Please enable JavaScript to view examples.
</noscript>

<template>
  <iframe src="https://codesandbox.io/p/devbox/3-components-0-7-rkjn3j?file=%2Fsrc%2Fmain.rs%3A39%2C10" width="100%" height="1000px" style="max-height: 100vh"></iframe>
</template>

```

<details>
<summary>CodeSandbox Source</summary>

```rust
use leptos::prelude::*;

// Composing different components together is how we build
// user interfaces. Here, we'll define a reusable <ProgressBar/>.
// You'll see how doc comments can be used to document components
// and their properties.

/// Shows progress toward a goal.
#[component]
fn ProgressBar(
    // Marks this as an optional prop. It will default to the default
    // value of its type, i.e., 0.
    #[prop(default = 100)]
    /// The maximum value of the progress bar.
    max: u16,
    // Will run `.into()` on the value passed into the prop.
    #[prop(into)]
    // `Signal<T>` is a wrapper for several reactive types.
    // It can be helpful in component APIs like this, where we
    // might want to take any kind of reactive value
    /// How much progress should be displayed.
    progress: Signal<i32>,
) -> impl IntoView {
    view! {
        <progress
            max={max}
            value=progress
        />
        <br/>
    }
}

#[component]
fn App() -> impl IntoView {
    let (count, set_count) = signal(0);

    let double_count = move || count.get() * 2;

    view! {
        <button
            on:click=move |_| {
                *set_count.write() += 1;
            }
        >
            "Click me"
        </button>
        <br/>
        // If you have this open in CodeSandbox or an editor with
        // rust-analyzer support, try hovering over `ProgressBar`,
        // `max`, or `progress` to see the docs we defined above
        <ProgressBar max=50 progress=count/>
        // Let's use the default max value on this one
        // the default is 100, so it should move half as fast
        <ProgressBar progress=count/>
        // Signal::derive creates a Signal wrapper from our derived signal
        // using double_count means it should move twice as fast
        <ProgressBar max=50 progress=Signal::derive(double_count)/>
    }
}

fn main() {
    leptos::mount::mount_to_body(App)
}
```

</details>
</preview>
