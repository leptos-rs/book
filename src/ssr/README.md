# Part 2: Server Side Rendering

The second part of the book is all about how to turn your beautiful UIs into full-stack Rust + Leptos powered websites and applications.

As you read in the last chapter, there are some limitations to using client-side rendered Leptos apps - over the next few chapters, you'll see how we can overcome those limitations
and get the best performance and SEO out of your Leptos apps.


```admonish info

When working with Leptos on the server side, you're free to choose either an officially supported Actix-web or Axum integrations, or one of our community supported choices. The full feature set of Leptos is available with the official choices, the community ones may support less. Check their documentation for details.

We have a variety of community supported choices, including WinterCG-compatible runtimes like Deno or Cloudflare. For Webassembly serverless runtimes we have Spin. There's also Viz and Pavex for more traditional server choices. This list is most likely incomplete, due to the nature of such lists. Writing an integration yourself isn't recommended as a beginner, but medium/advanced Rust users may wish to. Feel free to reach out if you have questions about that on our Discord or Github.

I'd recommend either Actix or Axum as a beginner, both are fully functional and choosing between them is a matter of personal preference. Axum is a bit more modular, and integrates well with the Tower ecosystem. Actix has been around longer and has a few more first party addon crates. There is no wrong choice there.
```
