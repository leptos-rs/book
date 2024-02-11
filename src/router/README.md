# Routing

## The Basics

Routing drives most websites. A router is the answer to the question, “Given this URL, what should appear on the page?”

A URL consists of many parts. For example, the URL `https://my-cool-blog.com/blog/search?q=Search#results` consists of

- a _scheme_: `https`
- a _domain_: `my-cool-blog.com`
- a **path**: `/blog/search`
- a **query** (or **search**): `?q=Search`
- a _hash_: `#results`

The Leptos Router works with the path and query (`/blog/search?q=Search`). Given this piece of the URL, what should the app render on the page?

## The Philosophy

In most cases, the path should drive what is displayed on the page. From the user’s perspective, for most applications, most major changes in the state of the app should be reflected in the URL. If you copy and paste the URL and open it in another tab, you should find yourself more or less in the same place.

In this sense, the router is really at the heart of the global state management for your application. More than anything else, it drives what is displayed on the page.

The router handles most of this work for you by mapping the current location to particular components.

## Compatibility Notes

If you develop a pure CSR application, and want to make it available using a traditional "dumb" webserver (this includes GitHub pages, Gitlab Pages, any server whose job is to expose a hierarchy of files and folders using HTTP(S)), you have to be aware that using the path to store application state will not work.  For this to work, you have to use a specialized hosting service (see [Deploying CSR Apps](../deployment/csr.md) for examples).

Using path to store application state does not make an app "not CSR any more", as once loaded the app will run as expected.  But you will only be able to load the app using its "base path", any attempt to use a URL containing state in the path with result in a `404 Not found` HTTP reply.

This is because a "dumb" webserver simply translates the URL path into a filesystem path.  State inside the path will be interpreted as additional folders below you web app top folder, and naturally will not be found.  Specialized handling of the HTTP request is needed to make the magic of "state stored in path" work.

The exact constraints on hosting are still to be described.  Until then, consider the following Rule-of-Thumb criteria:
- each app is hosted under its own domain, so its base path is `/` and the whole path is really available for storing state
- the hosting is advertized as tailored for web apps (so it will presumably always load your app from `/index.html`, and let the app interpret the path
