# Interlude: Styling

Anyone creating a website or application soon runs into the question of styling. For a small app, a single CSS file is probably plenty to style your user interface. But as an application grows, many developers find that plain CSS becomes increasingly hard to manage.

Some frontend frameworks (like Angular, Vue, and Svelte) provide built-in ways to scope your CSS to particular components, making it easier to manage styles across a whole application without styles meant to modify one small component having a global effect. Other frameworks (like React or Solid) don’t provide built-in CSS scoping, but rely on libraries in the ecosystem to do it for them. Leptos is in this latter camp: the framework itself has no opinions about CSS at all, but provides a few tools and primitives that allow others to build styling libraries.

Here are a few different approaches to styling your Leptos app, starting with plain CSS.

## Plain CSS

### Client-Side Rendering with Trunk

`trunk` can be used to bundle CSS files and images with your site. To do this, you can add them as Trunk assets by defining them in your `index.html` in the `<head>`. For example, to add a CSS file located at `style.css` you can add the tag `<link data-trunk rel="css" href="./style.css"/>`.

You can find more information in the Trunk documentation for [assets](https://trunkrs.dev/assets/).

### Server-Side Rendering with `cargo-leptos`

The `cargo-leptos` templates are configured by default to use SASS to bundle CSS files and output them at `/pkg/{project_name}.css`. If you want to load additional CSS files, you can do so either by importing them into that `style.scss` file, or by adding them to the `public` directory. (A file at `public/foo.css`, for example, is served at `/foo.css`.)

To load stylesheets in a component, you can use the [`Stylesheet`](https://docs.rs/leptos_meta/latest/leptos_meta/fn.Stylesheet.html) component.

## TailwindCSS: Utility-first CSS

[TailwindCSS](https://tailwindcss.com/) is a popular utility-first CSS library. It allows you to style your application by using inline utility classes, with a custom CLI tool that scans your files for Tailwind class names and bundles the necessary CSS.

This allows you to write components like this:

```rust
#[component]
fn Home() -> impl IntoView {
    let (count, set_count) = signal(0);

    view! {
        <main class="my-0 mx-auto max-w-3xl text-center">
            <h2 class="p-6 text-4xl">"Welcome to Leptos with Tailwind"</h2>
            <p class="px-10 pb-10 text-left">"Tailwind will scan your Rust files for Tailwind class names and compile them into a CSS file."</p>
            <button
                class="bg-sky-600 hover:bg-sky-700 px-5 py-3 text-white rounded-lg"
                on:click=move |_| *set_count.write() += 1
            >
                {move || if count.get() == 0 {
                    "Click me!".to_string()
                } else {
                    count.get().to_string()
                }}
            </button>
        </main>
    }
}
```

It can be a little complicated to set up the Tailwind integration at first, but you can check out our two examples of how to use Tailwind with a [client-side-rendered `trunk` application](https://github.com/leptos-rs/leptos/tree/main/examples/tailwind_csr) or with a [server-rendered `cargo-leptos` application](https://github.com/leptos-rs/leptos/tree/main/examples/tailwind_actix). `cargo-leptos` also has some [built-in Tailwind support](https://github.com/leptos-rs/cargo-leptos#site-parameters) that you can use as an alternative to Tailwind’s CLI.

## Stylers: Compile-time CSS Extraction

[Stylers](https://github.com/abishekatp/stylers) is a compile-time scoped CSS library that lets you declare scoped CSS in the body of your component. Stylers will extract this CSS at compile time into CSS files that you can then import into your app, which means that it doesn’t add anything to the WASM binary size of your application.

This allows you to write components like this:

```rust
use stylers::style;

#[component]
pub fn App() -> impl IntoView {
    let styler_class = style! { "App",
        ##two{
            color: blue;
        }
        div.one{
            color: red;
            content: raw_str(r#"\hello"#);
            font: "1.3em/1.2" Arial, Helvetica, sans-serif;
        }
        div {
            border: 1px solid black;
            margin: 25px 50px 75px 100px;
            background-color: lightblue;
        }
        h2 {
            color: purple;
        }
        @media only screen and (max-width: 1000px) {
            h3 {
                background-color: lightblue;
                color: blue
            }
        }
    };

    view! { class = styler_class,
        <div class="one">
            <h1 id="two">"Hello"</h1>
            <h2>"World"</h2>
            <h2>"and"</h2>
            <h3>"friends!"</h3>
        </div>
    }
}
```

## Stylance: Scoped CSS Written in CSS Files

Stylers lets you write CSS inline in your Rust code, extracts it at compile time, and scopes it. [Stylance](https://github.com/basro/stylance-rs) allows you to write your CSS in CSS files alongside your components, import those files into your components, and scope the CSS classes to your components.

This works well with the live-reloading features of `trunk` and `cargo-leptos` because edited CSS files can be updated immediately in the browser.

```rust
import_style!(style, "app.module.scss");

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class=style::jumbotron/>
    }
}
```

You can edit the CSS directly without causing a Rust recompile.

```css
.jumbotron {
  background: blue;
}
```

## Contributions Welcome

Leptos has no opinions on how you style your website or app, but we’re very happy to provide support to any tools you’re trying to create to make it easier. If you’re working on a CSS or styling approach that you’d like to add to this list, please let us know!
