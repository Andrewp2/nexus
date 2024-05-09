use leptos::{component, create_resource, view, IntoView, Params, SignalGet, SignalWith};
use leptos_router::{use_params, Params};

#[derive(Params, PartialEq, Clone)]
pub struct EmailVerificationParams {
    uuid: String,
}

#[component]
pub fn EmailVerificationAttempt() -> impl IntoView {
    let params = use_params::<EmailVerificationParams>();
    let uuid =
        move || params.with(|params| params.as_ref().map(|params| params.uuid.clone()).unwrap());

    let x = create_resource(
        || (),
        move |_| async move { crate::public::verify_email(uuid()).await },
    );

    let f = move || match x.get() {
        None => view! { <div>"hi there"</div> },
        Some(s) => match s {
            Ok(_) => view! { <div>"Verification was successful"</div> },
            Err(_) => view! { <div>"error oh no"</div> },
        },
    };

    view! { <div>{f}</div> }
}
