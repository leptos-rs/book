# Forms and Inputs

Forms and form inputs are an important part of interactive apps. There are two
basic patterns for interacting with inputs in Leptos, which you may recognize
if you’re familiar with React, SolidJS, or a similar framework: using **controlled**
or **uncontrolled** inputs.

## Controlled Inputs

In a "controlled input," the framework controls the state of the input
element. On every `input` event, it updates a local signal that holds the current
state, which in turn updates the `value` prop of the input.

There are two important things to remember:

1. The `input` event fires on (almost) every change to the element, while the
   `change` event fires (more or less) when you unfocus the input. You probably
   want `on:input`, but we give you the freedom to choose.
2. The `value` _attribute_ only sets the initial value of the input, i.e., it
   only updates the input up to the point that you begin typing. The `value`
   _property_ continues updating the input after that. You usually want to set
   `prop:value` for this reason. (The same is true for `checked` and `prop:checked`
   on an `<input type="checkbox">`.)

```rust
let (name, set_name) = signal("Controlled".to_string());

view! {
    <input type="text"
        // adding :target gives us typed access to the element
        // that is the target of the event that fires
        on:input:target=move |ev| {
            // .value() returns the current value of an HTML input element
            set_name.set(ev.target().value());
        }

        // the `prop:` syntax lets you update a DOM property,
        // rather than an attribute.
        prop:value=name
    />
    <p>"Name is: " {name}</p>
}
```

> #### Why do you need `prop:value`?
>
> Web browsers are the most ubiquitous and stable platform for rendering graphical user interfaces in existence. They have also maintained an incredible backwards compatibility over their three decades of existence. Inevitably, this means there are some quirks.
>
> One odd quirk is that there is a distinction between HTML attributes and DOM element properties, i.e., between something called an “attribute” which is parsed from HTML and can be set on a DOM element with `.setAttribute()`, and something called a “property” which is a field of the JavaScript class representation of that parsed HTML element.
>
> In the case of an `<input value=...>`, setting the `value` _attribute_ is defined as setting the initial value for the input, and setting `value` _property_ sets its current value. It may be easier to understand this by opening `about:blank` and running the following JavaScript in the browser console, line by line:
>
> ```js
> // create an input and append it to the DOM
> const el = document.createElement("input");
> document.body.appendChild(el);
>
> el.setAttribute("value", "test"); // updates the input
> el.setAttribute("value", "another test"); // updates the input again
>
> // now go and type into the input: delete some characters, etc.
>
> el.setAttribute("value", "one more time?");
> // nothing should have changed. setting the "initial value" does nothing now
>
> // however...
> el.value = "But this works";
> ```
>
> Many other frontend frameworks conflate attributes and properties, or create a special case for inputs that sets the value correctly. Maybe Leptos should do this too; but for now, I prefer giving users the maximum amount of control over whether they’re setting an attribute or a property, and doing my best to educate people about the actual underlying browser behavior rather than obscuring it.

### Simplifying Controlled Inputs with `bind:`

Adherence to Web standards and a clear division between “reading from a signal” and ”writing to a signal” are good, but creating
controlled inputs in this way can sometimes seem like more boilerplate than is really necessary.

Leptos also includes a special `bind:` syntax for inputs that allows you to automatically bind signals to inputs. They do exactly the same thing as the “controlled input” pattern above: create an event listener that updates the signal, and a dynamic property that reads from the signal. You can use `bind:value` for text inputs, `bind:checked` for checkboxes, and `bind:group` for radio button groups.

```rust
let (name, set_name) = signal("Controlled".to_string());
let email = RwSignal::new("".to_string());
let favorite_color = RwSignal::new("red".to_string());
let spam_me = RwSignal::new(true);

view! {
    <input type="text"
        bind:value=(name, set_name)
    />
    <input type="email"
        bind:value=email
    />
    <label>
        "Please send me lots of spam email."
        <input type="checkbox"
            bind:checked=spam_me
        />
    </label>
    <fieldset>
        <legend>"Favorite color"</legend>
        <label>
            "Red"
            <input
                type="radio"
                name="color"
                value="red"
                bind:group=favorite_color
            />
        </label>
        <label>
            "Green"
            <input
                type="radio"
                name="color"
                value="green"
                bind:group=favorite_color
            />
        </label>
        <label>
            "Blue"
            <input
                type="radio"
                name="color"
                value="bluee"
                bind:group=favorite_color
            />
        </label>
    </fieldset>
    <p>"Your favorite color is " {favorite_color} "."</p>
    <p>"Name is: " {name}</p>
    <p>"Email is: " {email}</p>
    <Show when=move || spam_me.get()>
        <p>"You’ll receive cool bonus content!"</p>
    </Show>
}
```

## Uncontrolled Inputs

In an "uncontrolled input," the browser controls the state of the input element.
Rather than continuously updating a signal to hold its value, we use a
[`NodeRef`](https://docs.rs/leptos/latest/leptos/tachys/reactive_graph/node_ref/struct.NodeRef.html) to access
the input when we want to get its value.

In this example, we only notify the framework when the `<form>` fires a `submit` event.
Note the use of the [`leptos::html`](https://docs.rs/leptos/latest/leptos/html/index.html) module, which provides a bunch of types for every HTML element.

```rust
let (name, set_name) = signal("Uncontrolled".to_string());

let input_element: NodeRef<html::Input> = NodeRef::new();

view! {
    <form on:submit=on_submit> // on_submit defined below
        <input type="text"
            value=name
            node_ref=input_element
        />
        <input type="submit" value="Submit"/>
    </form>
    <p>"Name is: " {name}</p>
}
```

The view should be pretty self-explanatory by now. Note two things:

1. Unlike in the controlled input example, we use `value` (not `prop:value`).
   This is because we’re just setting the initial value of the input, and letting
   the browser control its state. (We could use `prop:value` instead.)
2. We use `node_ref=...` to fill the `NodeRef`. (Older examples sometimes use `_ref`.
   They are the same thing, but `node_ref` has better rust-analyzer support.)

`NodeRef` is a kind of reactive smart pointer: we can use it to access the
underlying DOM node. Its value will be set when the element is rendered.

```rust
let on_submit = move |ev: SubmitEvent| {
    // stop the page from reloading!
    ev.prevent_default();

    // here, we'll extract the value from the input
    let value = input_element
        .get()
        // event handlers can only fire after the view
        // is mounted to the DOM, so the `NodeRef` will be `Some`
        .expect("<input> should be mounted")
        // `leptos::HtmlElement<html::Input>` implements `Deref`
        // to a `web_sys::HtmlInputElement`.
        // this means we can call`HtmlInputElement::value()`
        // to get the current value of the input
        .value();
    set_name.set(value);
};
```

Our `on_submit` handler will access the input’s value and use it to call `set_name`.
To access the DOM node stored in the `NodeRef`, we can simply call it as a function
(or using `.get()`). This will return `Option<leptos::HtmlElement<html::Input>>`, but we
know that the element has already been mounted (how else did you fire this event!), so
it's safe to unwrap here.

We can then call `.value()` to get the value out of the input, because `NodeRef`
gives us access to a correctly-typed HTML element.

Take a look at [`web_sys` and `HtmlElement`](../web_sys.md) to learn more about using a `leptos::HtmlElement`.
Also see the full CodeSandbox example at the end of this page.

## Special Cases: `<textarea>` and `<select>`

Two form elements tend to cause some confusion, in different ways.

### `<textarea>`

Unlike `<input>`, the `<textarea>` element does not support a `value` attribute in HTML.
Instead, it receives its initial value as a plain text node in its HTML children.

So if you’d like to server render an initial value, and have the value also react in the browser,
you can both pass it an initial text node as a child and use `prop:value` to
set its current value.

```rust
view! {
    <textarea
        prop:value=move || some_value.get()
        on:input:target=move |ev| some_value.set(ev.target().value())
    >
        {some_value}
    </textarea>
}
```

### `<select>`

The `<select>` element can likewise be controlled via a `value` property on the `<select>` itself,
which will select whichever `<option>` has that value.

```rust
let (value, set_value) = signal(0i32);
view! {
  <select
    on:change:target=move |ev| {
      set_value.set(ev.target().value().parse().unwrap());
    }
    prop:value=move || value.get().to_string()
  >
    <option value="0">"0"</option>
    <option value="1">"1"</option>
    <option value="2">"2"</option>
  </select>
  // a button that will cycle through the options
  <button on:click=move |_| set_value.update(|n| {
    if *n == 2 {
      *n = 0;
    } else {
      *n += 1;
    }
  })>
    "Next Option"
  </button>
}
```

```admonish sandbox title="Controlled vs uncontrolled forms CodeSandbox" collapsible=true

[Click to open CodeSandbox.](https://codesandbox.io/p/devbox/5-forms-0-7-l5hktg?file=%2Fsrc%2Fmain.rs&workspaceId=478437f3-1f86-4b1e-b665-5c27a31451fb)

<noscript>
  Please enable JavaScript to view examples.
</noscript>

<template>
  <iframe src="https://codesandbox.io/p/devbox/5-forms-0-7-l5hktg?file=%2Fsrc%2Fmain.rs&workspaceId=478437f3-1f86-4b1e-b665-5c27a31451fb" width="100%" height="1000px" style="max-height: 100vh"></iframe>
</template>

```

<details>
<summary>CodeSandbox Source</summary>

```rust
use leptos::{ev::SubmitEvent};
use leptos::prelude::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <h2>"Controlled Component"</h2>
        <ControlledComponent/>
        <h2>"Uncontrolled Component"</h2>
        <UncontrolledComponent/>
    }
}

#[component]
fn ControlledComponent() -> impl IntoView {
    // create a signal to hold the value
    let (name, set_name) = signal("Controlled".to_string());

    view! {
        <input type="text"
            // fire an event whenever the input changes
            // adding :target after the event gives us access to
            // a correctly-typed element at ev.target()
            on:input:target=move |ev| {
                set_name.set(ev.target().value());
            }

            // the `prop:` syntax lets you update a DOM property,
            // rather than an attribute.
            //
            // IMPORTANT: the `value` *attribute* only sets the
            // initial value, until you have made a change.
            // The `value` *property* sets the current value.
            // This is a quirk of the DOM; I didn't invent it.
            // Other frameworks gloss this over; I think it's
            // more important to give you access to the browser
            // as it really works.
            //
            // tl;dr: use prop:value for form inputs
            prop:value=name
        />
        <p>"Name is: " {name}</p>
    }
}

#[component]
fn UncontrolledComponent() -> impl IntoView {
    // import the type for <input>
    use leptos::html::Input;

    let (name, set_name) = signal("Uncontrolled".to_string());

    // we'll use a NodeRef to store a reference to the input element
    // this will be filled when the element is created
    let input_element: NodeRef<Input> = NodeRef::new();

    // fires when the form `submit` event happens
    // this will store the value of the <input> in our signal
    let on_submit = move |ev: SubmitEvent| {
        // stop the page from reloading!
        ev.prevent_default();

        // here, we'll extract the value from the input
        let value = input_element.get()
            // event handlers can only fire after the view
            // is mounted to the DOM, so the `NodeRef` will be `Some`
            .expect("<input> to exist")
            // `NodeRef` implements `Deref` for the DOM element type
            // this means we can call`HtmlInputElement::value()`
            // to get the current value of the input
            .value();
        set_name.set(value);
    };

    view! {
        <form on:submit=on_submit>
            <input type="text"
                // here, we use the `value` *attribute* to set only
                // the initial value, letting the browser maintain
                // the state after that
                value=name

                // store a reference to this input in `input_element`
                node_ref=input_element
            />
            <input type="submit" value="Submit"/>
        </form>
        <p>"Name is: " {name}</p>
    }
}

// This `main` function is the entry point into the app
// It just mounts our component to the <body>
// Because we defined it as `fn App`, we can now use it in a
// template as <App/>
fn main() {
    leptos::mount::mount_to_body(App)
}
```

</details>
</preview>
