use std::error::Error;

use leptos::*;
use leptos_router::*;

use crate::server::public::{Login, Signup};

#[component]
pub fn LoginPage() -> impl IntoView {
    let login = create_server_action::<Login>();
    let sign_up = create_server_action::<Signup>();
    let login_error: RwSignal<Option<Box<dyn Error>>> = create_rw_signal(Default::default());
    let signup_error = create_rw_signal(Default::default());
    view! {
        <div id="log-in-and-register-form">
            <ActionForm action=login error=login_error class="log-in-form">
                <h1>"Log in"</h1>
                <label>
                    "Email:" <br/>
                    <input type="email" placeholder="Email" maxlength="32" name="email" required/>
                </label>
                <br/>
                <label>
                    "Password:" <br/>
                    <input
                        type="password"
                        placeholder="Password"
                        name="password"
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
                <button type="submit" class="log-in-button">
                    "Log In"
                </button>
            </ActionForm>
            <ActionForm action=sign_up error=signup_error class="sign-up-form">
                <h1>"Sign Up"</h1>
                <label>
                    "Email:" <br/>
                    <input type="email" placeholder="Email" maxlength="64" name="email" required/>
                </label>
                <br/>
                <label>
                    "Password:" <br/>
                    <input
                        type="password"
                        placeholder="Password"
                        name="password"
                        required
                        minlength="10"
                    />
                </label>
                <br/>
                <label>
                    "Repeat password:" <br/>
                    <input
                        type="password"
                        placeholder="Repeat password"
                        name="password_confirmation"
                        required
                        minlength="10"
                    />
                </label>
                <br/>
                <button type="submit" class="sign-up-button">
                    "Sign Up"
                </button>
            </ActionForm>
        </div>
    }
}

