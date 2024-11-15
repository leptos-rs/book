# Part 2: Server Side Rendering

As you read in the last chapter, there are some limitations to using client-side rendered web applications. This second part of the book will discuss how to use server-side rendering to overcome these limitations and get the best performance and SEO out of your Leptos apps.


```admonish info

When working with Leptos on the server side, you’re free to choose either an officially supported Actix or Axum integrations, or one of our community supported choices. The full feature set of Leptos is available with the official choices, the community ones may support less. Check their documentation for details.

We have a variety of community supported choices, including WinterCG-compatible runtimes like Deno or Cloudflareand server-side WASM runtimes like Spin. Community-supported integrations for Viz and Pavex offer more traditional server choices. Writing an integration yourself isn't recommended as a beginner, but medium/advanced Rust users may wish to. Feel free to reach out if you have questions about that on our Discord or Github.

I'd recommend either Axum or Actix for beginners. Both are fully functional and choosing between them is a matter of personal preference. There is no wrong choice there, but if you’re looking for a recommendation, the Leptos team currently defaults to Axum for new projects.
```
