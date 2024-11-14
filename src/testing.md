# Testing Your Components

Testing user interfaces can be relatively tricky, but really important. This article
will discuss a couple principles and approaches for testing a Leptos app.

## 1. Test business logic with ordinary Rust tests

In many cases, it makes sense to pull the logic out of your components and test
it separately. For some simple components, there’s no particular logic to test, but
for many it’s worth using a testable wrapping type and implementing the logic in
ordinary Rust `impl` blocks.

For example, instead of embedding logic in a component directly like this:

```rust
#[component]
pub fn TodoApp() -> impl IntoView {
    let (todos, set_todos) = signal(vec![Todo { /* ... */ }]);
    // ⚠️ this is hard to test because it's embedded in the component
    let num_remaining = move || todos.read().iter().filter(|todo| !todo.completed).sum();
}
```

You could pull that logic out into a separate data structure and test it:

```rust
pub struct Todos(Vec<Todo>);

impl Todos {
    pub fn num_remaining(&self) -> usize {
        self.0.iter().filter(|todo| !todo.completed).sum()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_remaining() {
        // ...
    }
}

#[component]
pub fn TodoApp() -> impl IntoView {
    let (todos, set_todos) = signal(Todos(vec![Todo { /* ... */ }]));
    // ✅ this has a test associated with it
    let num_remaining = move || todos.read().num_remaining();
}
```

In general, the less of your logic is wrapped into your components themselves, the
more idiomatic your code will feel and the easier it will be to test.

## 2. Test components with end-to-end (`e2e`) testing

Our [`examples`](https://github.com/leptos-rs/leptos/tree/main/examples) directory has several examples with extensive end-to-end testing, using different testing tools.

The easiest way to see how to use these is to take a look at the test examples themselves:

### `wasm-bindgen-test` with [`counter`](https://github.com/leptos-rs/leptos/blob/main/examples/counter/tests/web.rs)

This is a fairly simple manual testing setup that uses the [`wasm-pack test`](https://rustwasm.github.io/wasm-pack/book/commands/test.html) command.

#### Sample Test

```rust
#[wasm_bindgen_test]
async fn clear() {
    let document = document();
    let test_wrapper = document.create_element("section").unwrap();
    let _ = document.body().unwrap().append_child(&test_wrapper);

    // start by rendering our counter and mounting it to the DOM
    // note that we start at the initial value of 10
    let _dispose = mount_to(
        test_wrapper.clone().unchecked_into(),
        || view! { <SimpleCounter initial_value=10 step=1/> },
    );

    // now we extract the buttons by iterating over the DOM
    // this would be easier if they had IDs
    let div = test_wrapper.query_selector("div").unwrap().unwrap();
    let clear = test_wrapper
        .query_selector("button")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>();

    // now let's click the `clear` button
    clear.click();

    // the reactive system is built on top of the async system, so changes are not reflected
    // synchronously in the DOM
    // in order to detect the changes here, we'll just yield for a brief time after each change,
    // allowing the effects that update the view to run
    tick().await;

    // now let's test the <div> against the expected value
    // we can do this by testing its `outerHTML`
    assert_eq!(div.outer_html(), {
        // it's as if we're creating it with a value of 0, right?
        let (value, _set_value) = signal(0);

        // we can remove the event listeners because they're not rendered to HTML
        view! {
            <div>
                <button>"Clear"</button>
                <button>"-1"</button>
                <span>"Value: " {value} "!"</span>
                <button>"+1"</button>
            </div>
        }
        // Leptos supports multiple backend renderers for HTML elements
        // .into_view() here is just a convenient way of specifying "use the regular DOM renderer"
        .into_view()
        // views are lazy -- they describe a DOM tree but don't create it yet
        // calling .build() will actually build the DOM elements
        .build()
        // .build() returned an ElementState, which is a smart pointer for
        // a DOM element. So we can still just call .outer_html(), which access the outerHTML on
        // the actual DOM element
        .outer_html()
    });

    // There's actually an easier way to do this...
    // We can just test against a <SimpleCounter/> with the initial value 0
    assert_eq!(test_wrapper.inner_html(), {
        let comparison_wrapper = document.create_element("section").unwrap();
        let _dispose = mount_to(
            comparison_wrapper.clone().unchecked_into(),
            || view! { <SimpleCounter initial_value=0 step=1/>},
        );
        comparison_wrapper.inner_html()
    });
}
```

### [Playwright with `counters`](https://github.com/leptos-rs/leptos/tree/main/examples/counters/e2e)

These tests use the common JavaScript testing tool Playwright to run end-to-end tests on the same example, using a library and testing approach familiar to many who have done frontend development before.

#### Sample Test

```js
test.describe("Increment Count", () => {
  test("should increase the total count", async ({ page }) => {
    const ui = new CountersPage(page);
    await ui.goto();
    await ui.addCounter();

    await ui.incrementCount();
    await ui.incrementCount();
    await ui.incrementCount();

    await expect(ui.total).toHaveText("3");
  });
});
```

### [Gherkin/Cucumber Tests with `todo_app_sqlite`](https://github.com/leptos-rs/leptos/blob/main/examples/todo_app_sqlite/e2e/README.md)

You can integrate any testing tool you’d like into this flow. This example uses Cucumber, a testing framework based on natural language.

```
@add_todo
Feature: Add Todo

    Background:
        Given I see the app

    @add_todo-see
    Scenario: Should see the todo
        Given I set the todo as Buy Bread
        When I click the Add button
        Then I see the todo named Buy Bread

    # @allow.skipped
    @add_todo-style
    Scenario: Should see the pending todo
        When I add a todo as Buy Oranges
        Then I see the pending todo
```

The definitions for these actions are defined in Rust code.

```rust
use crate::fixtures::{action, world::AppWorld};
use anyhow::{Ok, Result};
use cucumber::{given, when};

#[given("I see the app")]
#[when("I open the app")]
async fn i_open_the_app(world: &mut AppWorld) -> Result<()> {
    let client = &world.client;
    action::goto_path(client, "").await?;

    Ok(())
}

#[given(regex = "^I add a todo as (.*)$")]
#[when(regex = "^I add a todo as (.*)$")]
async fn i_add_a_todo_titled(world: &mut AppWorld, text: String) -> Result<()> {
    let client = &world.client;
    action::add_todo(client, text.as_str()).await?;

    Ok(())
}

// etc.
```

### Learning More

Feel free to check out the CI setup in the Leptos repo to learn more about how to use these tools in your own application. All of these testing methods are run regularly against actual Leptos example apps.
