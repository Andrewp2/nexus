use leptos::{
    component, create_server_action, create_signal, event_target_value, set_timeout, view, Action, IntoView, ServerFnError, WriteSignal
};
use leptos_router::ActionForm;
use web_sys::SubmitEvent;
use zxcvbn::zxcvbn;

use crate::{
    errors::NexusError,
    public::{Login, Signup},
};

#[component]
pub fn LoginAndSignup(
    login_action: Action<Login, Result<String, ServerFnError<NexusError>>>,
) -> impl IntoView {
    let (password, set_password) = create_signal("".to_string());
    let (login_disabled, set_login_disabled) = create_signal(false);
    let (signup_disabled, set_signup_disabled) = create_signal(false);
    let sign_up = create_server_action::<Signup>();
    let on_submit_login = move |(ev, set_login_disabled): (leptos::ev::SubmitEvent, WriteSignal<bool>)| {
        // stop the page from reloading!
        ev.prevent_default();
        set_login_disabled(true);
        set_timeout(move || {
            set_login_disabled(false);
        }, std::time::Duration::from_millis(500));
    };

    let on_submit_signup = move |(ev, set_signup_disabled): (leptos::ev::SubmitEvent, WriteSignal<bool>)| {
        // stop the page from reloading!
        ev.prevent_default();
        set_signup_disabled(true);
        set_timeout(move || {
            set_signup_disabled(false);
        }, std::time::Duration::from_millis(500));
    };
    view! {
        <div class="w-full flex flex-row box-border justify-evenly">
            <ActionForm action=login_action class="flex flex-col w-60" on:submit=move |e: SubmitEvent| on_submit_login((e, set_login_disabled))>
                <h1 class="text-2xl">"Log in"</h1>
                <div class="flex flex-col py-1">
                    <label>"Email:"</label>
                    <input
                        type="email"
                        placeholder="Email"
                        maxlength="64"
                        name="email"
                        required
                        class="text-gray-900"
                    />
                </div>
                <div class="flex flex-col py-1">
                    <label>"Password:" <br/></label>
                    <input

                        type="password"
                        placeholder="Password"
                        name="password"
                        required
                        minlength="10"
                        class="text-gray-900"
                    />
                </div>
                <div class="flex flex-row py-1 box-border">
                    <label class="pr-2">"Remember me?"</label>
                    <input type="checkbox" name="remember"/>
                </div>
                <input
                    type="submit"
                    class="w-max py-1 px-2 rounded-md bg-primary-color hover:bg-hover-accent-color glow-hover disabled:bg-slate-50"
                    disabled=login_disabled
                />
            </ActionForm>
            <ActionForm action=sign_up class="flex flex-col w-60" on:submit=move |e: SubmitEvent| on_submit_signup((e, set_signup_disabled))>
                <h1 class="text-2xl">"Sign Up"</h1>
                <div class="flex flex-col py-1">
                    <label>"Display name:"</label>
                    <input
                        type="text"
                        placeholder="Display Name"
                        maxlength="64"
                        name="display_name"
                        required
                        class="text-gray-900"
                    />
                </div>
                <div class="flex flex-col py-1">
                    <label>"Email:"</label>
                    <input
                        type="email"
                        placeholder="Email"
                        maxlength="64"
                        name="email"
                        required
                        class="text-gray-900"
                    />
                </div>
                <div class="flex flex-col py-1">
                    <label>"Password:"</label>
                    <input
                        type="password"
                        placeholder="Password"
                        name="password"
                        required
                        minlength="10"
                        on:input=move |ev| {
                            set_password(event_target_value(&ev));
                        }

                        prop:value=password
                        class="text-gray-900"
                    />
                </div>
                <div class="flex flex-col py-1">
                    <p>
                        "Password strength: "
                        {move || match zxcvbn(password().as_str(), &[]) {
                            Ok(o) => {
                                match o.score() {
                                    0 => "Very Weak",
                                    1 => "Weak",
                                    2 => "Fair",
                                    3 => "Good",
                                    4 => "Strong",
                                    _ => "Very Weak",
                                }
                            }
                            Err(_) => "Very Weak",
                        }}

                    </p>
                    <div class="flex justify-start items-center">
                        <div class="flex-1 rounded-lg bg-slate-400">
                            <div
                                // class="[min-width:4px] h-4 rounded-lg bg-slate-400"
                                class=move || {
                                    let password_strength = zxcvbn(password().as_str(), &[])
                                        .map_or(0u8, |e| e.score());
                                    let color = match password_strength {
                                        0 | 1 => "bg-red-600",
                                        2 => "bg-yellow-400",
                                        3 => "bg-lime-500",
                                        4 => "bg-lime-900",
                                        _ => "bg-red-600",
                                    };
                                    format!("w-full rounded-lg h-4 {color}")
                                }

                                style=move || {
                                    let password_strength = zxcvbn(password().as_str(), &[])
                                        .map_or(0u8, |e| e.score()) * 25;
                                    format!("width: {password_strength}%")
                                }
                            >
                            </div>
                        </div>
                    </div>
                </div>
                <div class="flex flex-col pt-1 pb-2">
                    <label>"Repeat password:" <br/></label>
                    <input
                        type="password"
                        placeholder="Repeat password"
                        name="password_confirmation"
                        required
                        minlength="10"
                    />

                </div>
                <input
                    type="submit"
                    class="w-max py-1 px-2 rounded-md bg-primary-color hover:bg-hover-accent-color glow-hover"
                    disabled=signup_disabled
                />
            </ActionForm>
        </div>
    }
}
