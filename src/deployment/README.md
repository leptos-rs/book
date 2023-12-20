# Deployment

There are as many ways to deploy a web application as there are developers, let alone applications. But there are a couple useful tips to keep in mind when deploying an app.

## General Advice

1. Remember: Always deploy Rust apps built in `--release` mode, not debug mode. This has a huge effect on both performance and binary size.
2. Test locally in release mode as well. The framework applies certain optimizations in release mode that it does not apply in debug mode, so it’s possible for bugs to surface at this point. (If your app behaves differently or you do encounter a bug, it’s likely a framework-level bug and you should open a GitHub issue with a reproduction.)
3. See the chapter on "Optimizing WASM Binary Size" for additional tips and tricks to further improve the time-to-interactive metric for your WASM app on first load.

> We asked users to submit their deployment setups to help with this chapter. I’ll quote from them below, but you can read the full thread [here](https://github.com/leptos-rs/leptos/issues/1152).

