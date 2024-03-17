# Leptos Developer Experience Improvements

There are a couple of things you can do to improve your experience of developing websites and apps with Leptos. You may want to take a few minutes and set up your environment to optimize your development experience, especially if you want to code along with the examples in this book.

## 1) Set up `console_error_panic_hook`

By default, panics that happen while running your WASM code in the browser just throw an error in the browser with an unhelpful message like `Unreachable executed` and a stack trace that points into your WASM binary.

With `console_error_panic_hook`, you get an actual Rust stack trace that includes a line in your Rust source code.

It's very easy to set up:

1. Run `cargo add console_error_panic_hook` in your project
2. In your main function, add `console_error_panic_hook::set_once();`

> If this is unclear, [click here for an example](https://github.com/leptos-rs/leptos/blob/main/examples/counter/src/main.rs#L4-L15).

Now you should have much better panic messages in the browser console!

## 2) Editor Autocompletion inside `#[component]` and `#[server]`

Because of the nature of macros (they can expand from anything to anything, but only if the input is exactly correct at that instant) it can be hard for rust-analyzer to do proper autocompletion and other support.

If you run into issues using these macros in your editor, you can explicitly tell rust-analyzer to ignore certain proc macros. For the `#[server]` macro especially, which annotates function bodies but doesn't actually transform anything inside the body of your function, this can be really helpful.

Starting in Leptos version 0.5.3, rust-analyzer support was added for the `#[component]` macro, but if you run into issues, you may want to add `#[component]` to the macro ignore list as well (see below).
Note that this means that rust-analyzer doesn't know about your component props, which may generate its own set of errors or warnings in the IDE.

VSCode `settings.json`:

```json
"rust-analyzer.procMacro.ignored": {
	"leptos_macro": [
        // optional:
		// "component",
		"server"
	],
}
```

VSCode with cargo-leptos `settings.json`:
```json
"rust-analyzer.procMacro.ignored": {
	"leptos_macro": [
        // optional:
		// "component",
		"server"
	],
},
"rust-analyzer.cargo.features": ["ssr"]
```

neovim with lspconfig:

```lua
require('lspconfig').rust_analyzer.setup {
  -- Other Configs ...
  settings = {
    ["rust-analyzer"] = {
      -- Other Settings ...
      procMacro = {
        ignored = {
            leptos_macro = {
                -- optional: --
                -- "component",
                "server",
            },
        },
      },
    },
  }
}
```

Helix, in `.helix/languages.toml`:

```toml
[[language]]
name = "rust"

[language-server.rust-analyzer]
config = { procMacro = { ignored =
    { leptos_macro =
        [
          # Optional:
          # "component",
          "server"
        ] } } }
```

## 3) Set up `leptosfmt` With Rust Analyzer (optional)

`leptosfmt` is a formatter for the Leptos `view!` macro (inside of which you'll typically write your UI code). Because the `view!` macro enables an 'RSX' (like JSX) style of writing your UI's, cargo-fmt has a harder time auto-formatting your code that's inside the `view!` macro. `leptosfmt` is a crate that solves your formatting issues and keeps your RSX-style UI code looking nice and tidy!

`leptosfmt` can be installed and used via the command line or from within your code editor:

First, install the tool with `cargo install leptosfmt`.

If you just want to use the default options from the command line, just run `leptosfmt ./**/*.rs` from the root of your project to format all the rust files using `leptosfmt`.

If you wish to set up your editor to work with `leptosfmt`, or if you wish to customize your `leptosfmt` experience, please see the instructions available on the [`leptosfmt` github repo's README.md page](https://github.com/bram209/leptosfmt).

Just note that it's recommended to set up your editor with `leptosfmt` on a per-workspace basis for best results.
