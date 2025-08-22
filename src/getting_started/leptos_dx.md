# Leptos Developer Experience Improvements

There are a couple of things you can do to improve your experience of developing websites and apps with Leptos. You may want to take a few minutes and set up your environment to optimize your development experience, especially if you want to code along with the examples in this book.

## 1) Set up `console_error_panic_hook`

By default, panics that happen while running your WASM code in the browser just throw an error in the browser with an unhelpful message like `Unreachable executed` and a stack trace that points into your WASM binary.

With `console_error_panic_hook`, you get an actual Rust stack trace that includes a line in your Rust source code.

It's very easy to set up:

1. Run `cargo add console_error_panic_hook` in your project
2. In your main function, add `console_error_panic_hook::set_once();`

> If this is unclear, [click here for an example](https://github.com/leptos-rs/leptos/blob/main/examples/counter/src/main.rs#L6).

Now you should have much better panic messages in the browser console!

## 2) Editor Autocompletion inside `#[component]` and `#[server]`

Because of the nature of macros (they can expand from anything to anything, but only if the input is exactly correct at that instant) it can be hard for rust-analyzer to do proper autocompletion and other support.

If you run into issues using these macros in your editor, you can explicitly tell rust-analyzer to ignore certain proc macros. For the `#[server]` macro especially, which annotates function bodies but doesn't actually transform anything inside the body of your function, this can be really helpful.

```admonish note 
 Starting in Leptos version 0.5.3, rust-analyzer support was added for the `#[component]` macro, but if you run into issues, you may want to add `#[component]` to the macro ignore list as well (see below).
Note that this means that rust-analyzer doesn't know about your component props, which may generate its own set of errors or warnings in the IDE.
```

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
// if code that is cfg-gated for the `ssr` feature is shown as inactive,
// you may want to tell rust-analyzer to enable the `ssr` feature by default
//
// you can also use `rust-analyzer.cargo.allFeatures` to enable all features
"rust-analyzer.cargo.features": ["ssr"]
```

Neovim:

```lua
vim.lsp.config('rust_analyzer', {
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
})
```

Helix, in `.helix/languages.toml`:

```toml
[[language]]
name = "rust"

[language-server.rust-analyzer]
config = { procMacro = { ignored = { leptos_macro = [
	# Optional:
	# "component",
	"server"
] } } }
```

Zed, in `settings.json`:

```json
{
  -- Other Settings ...
  "lsp": {
    "rust-analyzer": {
      "procMacro": {
        "ignored": [
          // optional:
          // "component",
          "server"
        ]
      }
    }
  }
}
```

SublimeText 3, under `LSP-rust-analyzer.sublime-settings` in `Goto Anything...` menu:

```json
// Settings in here override those in "LSP-rust-analyzer/LSP-rust-analyzer.sublime-settings"
{
  "rust-analyzer.procMacro.ignored": {
    "leptos_macro": [
      // optional:
      // "component",
      "server"
    ],
  },
}
```
## 3) Enable features in Rust-Analyzer for your Editor (optional)
By default, rust-analyzer will only run against the default features in your Rust project. Leptos uses different features to control compilation. For client side rendered projects, we use `csr` in different places, for server side rendered apps they can include `ssr` for server code and `hydrate` for code that we'll only run in the browser. 

How to enable these features varies by your IDE, we've listed some common ones below. If your IDE is not listed, you can usually find the setting by searching for `rust-analyzer.cargo.features` or `rust-analyzer.cargo.allFeatures`.

VSCode, in `settings.json`:
```json
{
  "rust-analyzer.cargo.allFeatures": true,  // Enable all features
}
```

Neovim, in `init.lua`:
```lua
vim.lsp.config('rust_analyzer', {
  settings = {
    ["rust-analyzer"] = {
      cargo = {
        features = "all", -- Enable all features
      },
    },
  }
})

```
helix, in `.helix/languages.toml` or per project in `.helix/config.toml`:
```toml
[[language]]
name = "rust"

[language-server.rust-analyzer.config.cargo]
allFeatures = true
```

Zed, in `settings.json`:

```json
{
  -- Other Settings ...
  "lsp": {
    "rust-analyzer": {
      "initialization_options": {
        "cargo": {
          "allFeatures": true // Enable all features
        }
      }
	}
  }
}
```

SublimeText 3,in the user settings for LSP-rust-analyzer-settings.json
```json
 {
        "settings": {
            "LSP": {
                "rust-analyzer": {
                    "settings": {
                        "cargo": {
                            "features": "all"
                        }
                    }
                }
            }
        }
    }
```


## 4) Set up `leptosfmt` (optional)

`leptosfmt` is a formatter for the Leptos `view!` macro (inside of which you'll typically write your UI code). Because the `view!` macro enables an 'RSX' (like JSX) style of writing your UI's, cargo-fmt has a harder time auto-formatting your code that's inside the `view!` macro. `leptosfmt` is a crate that solves your formatting issues and keeps your RSX-style UI code looking nice and tidy!

`leptosfmt` can be installed and used via the command line or from within your code editor:

First, install the tool with `cargo install leptosfmt`.

If you just want to use the default options from the command line, just run `leptosfmt ./**/*.rs` from the root of your project to format all the rust files using `leptosfmt`.

### Run automatically in Rust Analyzer IDEs

If you wish to set up your editor to work with `leptosfmt`, or if you wish to customize your `leptosfmt` experience, please see the instructions available on the [`leptosfmt` github repo's README.md page](https://github.com/bram209/leptosfmt).

Just note that it's recommended to set up your editor with `leptosfmt` on a per-workspace basis for best results.

### Run automatically in RustRover

Unfortunately, RustRover does not support Rust Analyzer, so a different approach is required in order to automatically
run `leptosfmt`. One way is to use the [FileWatchers](https://plugins.jetbrains.com/plugin/7177-file-watchers) plugin
with the below configuration:

- Name: Leptosfmt
- File type: Rust files
- Program: `/path/to/leptosfmt` (can simply be `leptosfmt` if it's in your `$PATH` environment variable)
- Arguments: `$FilePath$`
- Output paths to refresh: `$FilePath$`


## 5) Use `--cfg=erase_components` during development

Leptos 0.7 made a number of changes to the renderer that relied more heavily on the type system. For larger projects, this can lead to slower compile times. Most of the slowdown in compile times can be alleviated by using the custom configuration flag `--cfg=erase_components` during development. (This erases some of that type information to reduce the amount of work done and debug info emitted by the compiler, at the expense of additional binary size and runtime cost, so it’s best not to use it in release mode.) 

As of cargo-leptos v0.2.40, this is automatically enabled for you in development mode. If you are using trunk, not using cargo-leptos, or want to enable it for non-dev uses, you can set this easily in the command line (`RUSTFLAGS="--cfg erase_components" trunk serve` or `RUSTFLAGS="--cfg erase_components" cargo leptos watch`), or in your `.cargo/config.toml`:
```toml
# use your own native target
[target.aarch64-apple-darwin]
rustflags = [
  "--cfg",
  "erase_components",
]

[target.wasm32-unknown-unknown]
rustflags = [
   "--cfg",
   "erase_components",
]
```
