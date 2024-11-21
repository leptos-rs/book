#![cfg(test)]
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

#[component]
fn Home() -> impl IntoView {
    view! { "home" }
}
#[component]
fn Users() -> impl IntoView {
    view! { "Users" }
}
#[component]
fn UserProfile() -> impl IntoView {
    view! { "UserProfile" }
}

#[test]
fn test_s1() {
    #[component]
    pub fn App() -> impl IntoView {
        view! {
            // ANCHOR: s1
            <Routes fallback=|| "Not found.">
                <Route path=path!("/") view=Home />
                <Route path=path!("/users") view=Users />
                <Route path=path!("/users/:id") view=UserProfile />
                <Route path=path!("/*any") view=|| view! { <h1>"Not Found"</h1> } />
            </Routes>
            // ANCHOR_END: s1
        }
    }
}

#[test]
fn test_s2() {
    #[component]
    pub fn App() -> impl IntoView {
        view! {
            // ANCHOR: s2
            <Routes fallback=|| "Not found.">
                <Route path=path!("/") view=Home />
                <ParentRoute path=path!("/users") view=Users>
                    <Route path=path!(":id") view=UserProfile />
                </ParentRoute>
                <Route path=path!("/*any") view=|| view! { <h1>"Not Found"</h1> } />
            </Routes>
            // ANCHOR_END: s2
        }
    }
}

#[test]
fn test_s3() {
    #[component]
    pub fn App() -> impl IntoView {
        view! {
            // ANCHOR: s3
            <Routes fallback=|| "Not found.">
                <Route path=path!("/users") view=Users/>
                <Route path=path!("/users/:id") view=UserProfile/>
            </Routes>
            // ANCHOR_END: s3
        }
    }
}

#[test]
fn test_s4() {
    #[component]
    pub fn App() -> impl IntoView {
        view! {
            // ANCHOR: s4
            <Routes fallback=|| "Not found.">
                <ParentRoute path=path!("/users") view=Users>
                  <Route path=path!(":id") view=UserProfile/>
                </ParentRoute>
            </Routes>
            // ANCHOR_END: s4
        }
    }
}

#[test]
fn test_s5() {
    #[component]
    pub fn App() -> impl IntoView {
        view! {
        // ANCHOR: s5
            <Routes>
                <ParentRoute path="/users" view=Users>
                    <Route path=":id" view=UserProfile/>
                    <Route path="" view=NoUser/>
                </ParentRoute>
            </Routes>
        // ANCHOR_END: s5
        }
    }
}

#[test]
fn test_s_next() {
    #[component]
    pub fn App() -> impl IntoView {
        view! {
            // ANCHOR: s_next

            // ANCHOR_END: s_next
        }
    }
}


