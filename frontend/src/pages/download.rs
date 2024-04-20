use leptos::{component, view, IntoView, ReadSignal, WriteSignal};
use leptos_router::A;

#[component]
pub fn Download(logged_in: ReadSignal<bool>, set_logged_in: WriteSignal<bool>) -> impl IntoView {
    view! { <h1>"Download"</h1> }
}
