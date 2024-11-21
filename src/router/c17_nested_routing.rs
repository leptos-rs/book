#![cfg(test)]
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

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

#[component]
fn Home() -> impl IntoView {
    view! {
        "home"
    }
}
#[component]
fn Users() -> impl IntoView {
    view! {
        "Users"
    }
}
#[component]
fn UserProfile() -> impl IntoView {
    view! {
        "UserProfile"
    }
}
}

