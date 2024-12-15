# Working with Signals

So far we’ve used some simple examples of using [`signal`](https://docs.rs/leptos/latest/leptos/reactive/signal/fn.signal.html), which returns a [`ReadSignal`](https://docs.rs/leptos/latest/leptos/reactive/signal/struct.ReadSignal.html) getter and a [`WriteSignal`](https://docs.rs/leptos/latest/leptos/reactive/signal/struct.WriteSignal.html) setter.

## Getting and Setting

There are a few basic signal operations:

### Getting

1. [`.read()`](https://docs.rs/leptos/latest/leptos/reactive/signal/struct.ReadSignal.html#impl-Read-for-T) returns a read guard which dereferences to the value of the signal, and tracks any future changes to the value of the signal reactively. Note that you cannot update the value of the signal until this guard is dropped, or it will cause a runtime error.
1. [`.with()`](https://docs.rs/leptos/latest/leptos/reactive/signal/struct.ReadSignal.html#impl-With-for-T) takes a function, which receives the current value of the signal by reference (`&T`), and tracks the signal.
1. [`.get()`](https://docs.rs/leptos/latest/leptos/reactive/signal/struct.ReadSignal.html#impl-Get-for-T) clones the current value of the signal and tracks further changes to the value.

`.get()` is the most common method of accessing a signal. `.read()` is useful for methods that take an immutable reference, without cloning the value (`my_vec_signal.read().len()`). `.with()` is useful if you need to do more with that reference, but want to make sure you don’t hold onto the lock longer than you need.

### Setting

1. [`.write()`](https://docs.rs/leptos/latest/leptos/reactive/signal/struct.WriteSignal.html#impl-Write-for-WriteSignal%3CT,+S%3E) returns a write guard which is a mutable references to the value of the signal, and notifies any subscribers that they need to update. Note that you cannot read from the value of the signal until this guard is dropped, or it will cause a runtime error.
1. [`.update()`](https://docs.rs/leptos/latest/leptos/reactive/signal/struct.WriteSignal.html#impl-Update-for-T) takes a function, which receives a mutable reference to the current value of the signal (`&mut T`), and notifies subscribers. (`.update()` doesn’t return the value returned by the closure, but you can use [`.try_update()`](https://docs.rs/leptos/latest/leptos/trait.SignalUpdate.html#tymethod.try_update) if you need to; for example, if you’re removing an item from a `Vec<_>` and want the removed item.)
1. [`.set()`](https://docs.rs/leptos/latest/leptos/reactive/signal/struct.WriteSignal.html#impl-Set-for-T) replaces the current value of the signal and notifies subscribers.

`.set()` is most common for setting a new value; `.write()` is very useful for updating a value in place. Just as is the case with `.read()` and `.with()`, `.update()` can be useful when you want to avoid the possibility of holding on the write lock longer than you intended to.

```admonish note
These traits are based on trait composition and provided by blanket implementations. For example, `Read` is implemented for any type that implements `Track` and `ReadUntracked`. `With` is implemented for any type that implements `Read`. `Get` is implemented for any type that implements `With` and `Clone`. And so on.

Similar relationships exist for `Write`, `Update`, and `Set`.

This is worth noting when reading docs: if you only see `ReadUntracked` and `Track` as implemented traits, you will still be able to use `.with()`, `.get()` (if `T: Clone`), and so on.
```

## Working with Signals

You might notice that `.get()` and `.set()` can be implemented in terms of `.read()` and `.write()`, or `.with()` and `.update()`. In other words, `count.get()` is identical with `count.with(|n| n.clone())` or `count.read().clone()`, and `count.set(1)` is implemented by doing `count.update(|n| *n = 1)` or `*count.write() = 1`.

But of course, `.get()` and `.set()` are nicer syntax.

However, there are some very good use cases for the other methods.

For example, consider a signal that holds a `Vec<String>`.

```rust
let (names, set_names) = signal(Vec::new());
if names.get().is_empty() {
	set_names(vec!["Alice".to_string()]);
}
```

In terms of logic, this is simple enough, but it’s hiding some significant inefficiencies. Remember that `names.get().is_empty()` clones the value. This means we clone the whole `Vec<String>`, run `is_empty()`, and then immediately throw away the clone.

Likewise, `set_names` replaces the value with a whole new `Vec<_>`. This is fine, but we might as well just mutate the original `Vec<_>` in place.

```rust
let (names, set_names) = signal(Vec::new());
if names.read().is_empty() {
	set_names.write().push("Alice".to_string());
}
```

Now our function simply takes `names` by reference to run `is_empty()`, avoiding that clone, and then mutates the `Vec<_>` in place.

## Nightly Syntax

When using the `nightly` feature and `nightly` syntax, calling a `ReadSignal` as a function is syntax sugar for `.get()`. Calling a `WriteSignal` as a function is syntax sugar for `.set()`. So

```rust
let (count, set_count) = signal(0);
set_count(1);
logging::log!(count());
```

is the same as

```rust
let (count, set_count) = signal(0);
set_count.set(1);
logging::log!(count.get());
```

This is not just syntax sugar, but makes for a more consistent API by making signals semantically the same thing as functions: see the [Interlude](./interlude_functions.md).

## Making signals depend on each other

Often people ask about situations in which some signal needs to change based on some other signal’s value. There are three good ways to do this, and one that’s less than ideal but okay under controlled circumstances.

### Good Options

**1) B is a function of A.** Create a signal for A and a derived signal or memo for B.

```rust
// A
let (count, set_count) = signal(1);
// B is a function of A
let derived_signal_double_count = move || count.get() * 2;
// B is a function of A
let memoized_double_count = Memo::new(move |_| count.get() * 2);
```

> For guidance on whether to use a derived signal or a memo, see the docs for [`Memo`](https://docs.rs/leptos/latest/leptos/reactive/computed/struct.Memo.html)

**2) C is a function of A and some other thing B.** Create signals for A and B and a derived signal or memo for C.

```rust
// A
let (first_name, set_first_name) = signal("Bridget".to_string());
// B
let (last_name, set_last_name) = signal("Jones".to_string());
// C is a function of A and B
let full_name = move || format!("{} {}", &*first_name.read(), &*last_name.read()));
```

**3) A and B are independent signals, but sometimes updated at the same time.** When you make the call to update A, make a separate call to update B.

```rust
// A
let (age, set_age) = signal(32);
// B
let (favorite_number, set_favorite_number) = signal(42);
// use this to handle a click on a `Clear` button
let clear_handler = move |_| {
  // update both A and B
  set_age.set(0);
  set_favorite_number.set(0);
};
```

### If you really must...

**4) Create an effect to write to B whenever A changes.** This is officially discouraged, for several reasons:
a) It will always be less efficient, as it means every time A updates you do two full trips through the reactive process. (You set A, which causes the effect to run, as well as any other effects that depend on A. Then you set B, which causes any effects that depend on B to run.)
b) It increases your chances of accidentally creating things like infinite loops or over-re-running effects. This is the kind of ping-ponging, reactive spaghetti code that was common in the early 2010s and that we try to avoid with things like read-write segregation and discouraging writing to signals from effects.

In most situations, it’s best to rewrite things such that there’s a clear, top-down data flow based on derived signals or memos. But this isn’t the end of the world.

> I’m intentionally not providing an example here. Read the [`Effect`](https://docs.rs/leptos/latest/leptos/reactive/effect/struct.Effect.html) docs to figure out how this would work.
