# Appendix: The Life Cycle of a Signal

Three questions commonly arise at the intermediate level when using Leptos:
1. How can I connect to the component lifecycle, running some code when a component mounts or unmounts?
2. How do I know when signals are disposed, and why do I get an occasional panic when trying to access a disposed signal?
3. How is it possible that signals are `Copy` and can be moved into closures and other structures without being explicitly cloned?

The answers to these three questions are closely inter-related, and are each somewhat complicated. This appendix will try to give you the context for understanding the answers, so that you can reason correctly about your application's code and how it runs.

## The Component Tree vs. The Decision Tree

Consider the following simple Leptos app:

```rust
use leptos::logging::log;
use leptos::*;

#[component]
pub fn App() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <button on:click=move |_| set_count.update(|n| *n += 1)>"+1"</button>
        {move || if count() % 2 == 0 {
            view! { <p>"Even numbers are fine."</p> }.into_view()
        } else {
            view! { <InnerComponent count/> }.into_view()
        }}
    }
}

#[component]
pub fn InnerComponent(count: ReadSignal<usize>) -> impl IntoView {
    create_effect(move |_| {
        log!("count is odd and is {}", count());
    });

    view! {
        <OddDuck/>
        <p>{count}</p>
    }
}

#[component]
pub fn OddDuck() -> impl IntoView {
    view! {
        <p>"You're an odd duck."</p>
    }
}
```

All it does is show a counter button, and then one message if it's even, and a different message if it's odd. If it's odd, it also logs the values in the console.

One way to map out this simple application would be to draw a tree of nested components:
```
App 
|_ InnerComponent
   |_ OddDuck
```

Another way would be to draw the tree of decision points:
```
root
|_ is count even?
   |_ yes
   |_ no
```

If you combine the two together, you'll notice that they don't map onto one another perfectly. The decision tree slices the view we created in `InnerComponent` into three pieces, and combines part of `InnerComponent` with the `OddDuck` component:
```
DECISION            COMPONENT           DATA    SIDE EFFECTS
root                <App/>              (count) render <button>
|_ is count even?   <InnerComponent/>
   |_ yes                                       render even <p>
   |_ no                                        start logging the count 
                    <OddDuck/>                  render odd <p> 
                                                render odd <p> (in <InnerComponent/>!)
```

Looking at this table, I notice the following things:
1. The component tree and the decision tree don't match one another: the "is count even?" decision splits `<InnerComponent/>` into three parts (one that never changes, one if even, one if odd), and merges one of these with the `<OddDuck/>` component. 
2. The decision tree and the list of side effects correspond perfectly: each side effect is created at a specific decision point.
3. The decision tree and the tree of data also line up. It's hard to see with only one signal in the table, but unlike a component, which is a function that can include multiple decisions or none, a signal is always created at a specific line in the tree of decisions.

Here's the thing: The structure of your data and the structure of side effects affect the actual functionality of your application. The structure of your components is just a convenience of authoring. You don't care, and you shouldn't care, which component rendered which `<p>` tag, or which component created the effect to log the values. All that matters is that they happen at the right times.

In Leptos, *components do not exist.* That is to say: You can write your application as a tree of components, because that's convenient, and we provide some debugging tools and logging built around components, because that's convenient too. But your components do not exist at runtime: Components are not a unit of change detection or of rendering. They are simply function calls. You can write your whole application in one big component, or split it into a hundred components, and it does not affect the runtime behavior, because components don't really exist.

The decision tree, on the other hand, *does exist*. And it's really important!

## The Decision Tree, Rendering, and Ownership

Every decision point is some kind of reactive statement: a signal or a function that can change over time. When you pass a signal or a function into the renderer, it automatically wraps it in an effect that subscribes to any signals it contains, and updates the view accordingly over time.

This means that when your application is rendered, it creates a tree of nested effects that perfectly mirrors the decision tree. In pseudo-code:
```rust
// root
let button = /* render the <button> once */;

// the renderer wraps an effect around the `move || if count() ...`
create_effect(|_| {
    if count() % 2 == 0 {
        let p = /* render the even <p> */;
    } else {
        // the user created an effect to log the count
        create_effect(|_| {
            log!("count is odd and is {}", count());
        });

        let p1 = /* render the <p> from OddDuck */;
        let p2 = /* render the second <p> */ 

        // the renderer creates an effect to update the second <p>
        create_effect(|_| {
            // update the content of the <p> with the signal
            p2.set_text_content(count.get());
        });
    }
})
```

Each reactive value is wrapped in its own effect to update the DOM, or run any other side effects of changes to signals. But you don't need these effects to keep running forever. For example, when `count` switches from an odd number back to an even number, the second `<p>` no longer exists, so the effect to keep updating it is no longer useful. Instead of running forever, effects are canceled when the decision that created them changes. In other words, and more precisely: effects are canceled whenever the effect that was running when they were created re-runs. If they were created in a conditional branch, and re-running the effect goes through the same branch, the effect will be created again: if not, it will not.

From the perspective of the reactive system itself, your application's "decision tree" is really a reactive "ownership tree." Simply put, a reactive "owner" is the effect or memo that is currently running. It owns effects created within it, they own their own children, and so on. When an effect is going to re-run, it first "cleans up" its children, then runs again.

So far, this model is shared with the reactive system as it exists in JavaScript frameworks like S.js or Solid, in which the concept of ownership exists to automatically cancel effects.

What Leptos adds is that we add a second, similar meaning to ownership: a reactive owner not only owns its child effects, so that it can cancel them; it also owns its signals (memos, etc.) so that it can dispose of them.

## Ownership and the `Copy` Arena

This is the innovation that allows Leptos to be usable as a Rust UI framework. Traditionally, managing UI state in Rust has been hard, because UI is all about shared mutability. (A simple counter button is enough to see the problem: You need both immutable access to set the text node showing the counter's value, and mutable access in the click handler, and every Rust UI framework is designed around the fact that Rust is designed to prevent exactly that!) Using something like an event handler in Rust traditionally relies on primitives for communicating via shared memory with interior mutability (`Rc<RefCell<_>>`, `Arc<Mutex<_>>`) or for shared memory by communicating via channels, either of which often requires explicit `.clone()`ing to be moved into an event listener. This is kind of fine, but also an enormous inconvenience.

Leptos has always used a form of arena allocation for signals instead. A signal itself is essentially an index into a data structure that's held elsewhere. It's a cheap-to-copy integer type that does not do reference counting on its own, so it can be copied around, moved into event listeners, etc. without explicit cloning.

Instead of Rust lifetimes or reference counting, the life cycles of these signals are determined by the ownership tree.

Just as all effects belong to an owning parent effect, and the children are canceled when the owner reruns, so too all signals belong to an owner, and are disposed of when the parent reruns. 

In most cases, this is completely fine. Imagine that in our example above, `<OddDuck/>` created some other signal that it used to update part of its UI. In most cases, that signal will be used for local state in that component, or maybe passed down as a prop to another component. It's unusual for it to be hoisted up out of the decision tree and used somewhere else in the application. When the `count` switches back to an even number, it is no longer needed and can be disposed.

However, this means there are two possible issues that can arise.

### Signals can be used after they are disposed 

The `ReadSignal` or `WriteSignal` that you hold is just an integer: say, 3 if it's the 3rd signal in the application. (As always, the reality is a bit more complicated, but not much.) You can copy that number all over the place and use it to say, "Hey, get me signal 3." When the owner cleans up, the *value* of signal 3 will be invalidated; but the number 3 that you've copied all over the place can't be invalidated. (Not without a whole garbage collector!) That means that if you push signals back "up" the decision tree, and store them somewhere conceptually "higher" in your application than they were created, they can be accessed after being disposed.

If you try to *update* a signal after it was disposed, nothing bad really happens. The framework will just warn you that you tried to update a signal that no longer exists. But if you try to *access* one, there's no coherent answer other than panicking: there is no value that could be returned. (There are `try_` equivalents to the `.get()` and `.with()` methods that will simply return `None` if a signal has been disposed).

### Signals can be leaked if you create them in a higher scope and never dispose of them

The opposite is also true, and comes up particularly when working with collections of signals, like an `RwSignal<Vec<RwSignal<_>>>`. If you create a signal at a higher level, and pass it down to a component at a lower level, it is not disposed until the higher-up owner is cleaned up. 

For example, if you have a todo app that creates a new `RwSignal<Todo>` for each todo, stores it in an `RwSignal<Vec<RwSignal<Todo>>>`, and then passes it down to a `<Todo/>`, that signal is not automatically disposed when you remove the todo from the list, but must be manually disposed, or it will "leak" for as long as its owner is still alive. (See the [TodoMVC example](https://github.com/leptos-rs/leptos/blob/main/examples/todomvc/src/lib.rs#L77-L85) for more discussion.) 

This is only an issue when you create signals, store them in a collection, and remove them from the collection without manually disposing of them as well.

## Connecting the Dots

The answers to the questions we started with should probably make some sense now.

### Component Life-Cycle

There is no component life-cycle, because components don't really exist. But there is an ownership lifecycle, and you can use it to accomplish the same things:
- *before mount*: simply running code in the body of a component will run it "before the component mounts"
- *on mount*: `create_effect` runs a tick after the rest of the component, so it can be useful for effects that need to wait for the view to be mounted to the DOM. 
- *on unmount*: You can use `on_cleanup` to give the reactive system code that should run while the current owner is cleaning up, before running again. Because an owner is around a "decision," this means that `on_cleanup` will run when your component unmounts: if something can unmount, the renderer must have created an effect that's unmounting it!

### Issues with Disposed Signals

Generally speaking, problems can only arise here if you are creating a signal lower down in the ownership tree and storing it somewhere higher up. If you run into issues here, you should instead "hoist" the signal creation up into the parent, and then pass the created signals down—making sure to dispose of them on removal, if needed!

### `Copy` signals

The whole system of `Copy`able wrapper types (signals, `StoredValue`, and so on) uses the ownership tree as a close approximation of the life-cycle of different parts of your UI. In effect, it parallels the Rust language's system of lifetimes based on blocks of code with a system of lifetimes based on sections of UI. This can't always be perfectly checked at compile time, but overall we think it's a net positive.
