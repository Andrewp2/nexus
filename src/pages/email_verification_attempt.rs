use leptos::*;
use leptos_router::*;

#[derive(Params, PartialEq, Clone)]
pub struct EmailVerificationParams {
    uuid: String,
}

#[component]
pub fn EmailVerificationAttempt() -> impl IntoView {
    let params = use_params::<EmailVerificationParams>();
    let uuid = move || {
        params.with(|params| {
            params
                .as_ref()
                .map(|params| params.uuid.clone())
                .unwrap_or_default()
        })
    };

    let x = create_resource(
        || (),
        move |_| async move { crate::server::public::verify_email(uuid()).await },
    );

    let f = move || match x.get() {
        None => view! { <div>hi there</div> },
        Some(s) => match s {
            Ok(_) => view! { <div>Verification was successful</div> },
            Err(e) => view! { <div>error oh no</div> },
        },
    };

    view! { <div>{f}</div> }
}

