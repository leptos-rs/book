# Working with `async`

So far we’ve only been working with synchronous user interfaces: You provide some input,
the app immediately processes it and updates the interface. This is great, but is a tiny
subset of what web applications do. In particular, most web apps have to deal with some kind of asynchronous data loading, usually loading something from an API.

Asynchronous data is notoriously hard to integrate with the synchronous parts of your code because of problems of “function coloring.”

In the following chapters, we’ll see a few reactive primitives for working with async data. But it’s important to note at the very beginning: If you just want to do some asynchronous work, Leptos provides a cross-platform [`spawn_local`](https://docs.rs/leptos/0.7.0-gamma3/leptos/task/fn.spawn_local.html) function that makes it easy to run a `Future`. If one of the primitives that’s discussd in the rest of this section doesn’t seem to do what you want, consider combining `spawn_local` with setting a signal.

While the primitives to come are very useful, and even necessary in some cases, people sometimes run into situations in which they really just need to spawn a task and wait for it to finish before doing something else. Use `spawn_local` in those situations!
