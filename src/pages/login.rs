use std::error::Error;

use leptos::*;
use leptos_router::*;

use crate::server::public::Login;

#[component]
pub fn LoginPage() -> impl IntoView {
    let login = create_server_action::<Login>();
    let error: RwSignal<Option<Box<dyn Error>>> = create_rw_signal(Default::default());
    view! {
        <ActionForm action=login error=error>
            <h1>"Log in"</h1>
            <label>
                "Email:" <br/>
                <input
                    type="email"
                    placeholder="Email"
                    maxlength="32"
                    name="email"
                    class="auth-input"
                    required
                />
            </label>
            <br/>
            <label>
                "Password:" <br/>
                <input
                    type="password"
                    placeholder="Password"
                    name="password"
                    class="auth-input"
                    required
                    minlength="10"
                />
            </label>
            <br/>
            <label>
                <input type="checkbox" name="remember" class="auth-input"/>
                "Remember me?"
            </label>
            <br/>
            <button type="submit" class="button">
                "Log In"
            </button>
        </ActionForm>
    }
}

