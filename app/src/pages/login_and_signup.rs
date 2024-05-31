use leptos::{
    component, create_effect, create_server_action, create_signal, event_target_value, view,
    Action, IntoView, ReadSignal, ServerFnError, SignalGet, SignalSet, SignalWith, WriteSignal,
};
use leptos_router::ActionForm;
use zxcvbn::zxcvbn;

use crate::{
    errors::NexusError,
    public::{Login, Signup},
};

#[component]
pub fn LoginAndSignup(
    login_action: Action<Login, Result<(), ServerFnError<NexusError>>>,
) -> impl IntoView {
    let (password, set_password) = create_signal("".to_string());
    let sign_up = create_server_action::<Signup>();
    view! {
        <div class="w-full flex flex-row box-border justify-evenly">
            <ActionForm action=login_action class="flex flex-col w-60">
                <h1 class="text-2xl">"Log in"</h1>
                <div class="flex flex-col py-1">
                    <label>"Email:"</label>
                    <input type="email" placeholder="Email" maxlength="64" name="email" required/>
                </div>
                <div class="flex flex-col py-1">
                    <label>"Password:" <br/></label>
                    <input

                        type="password"
                        placeholder="Password"
                        name="password"
                        required
                        minlength="10"
                    />
                </div>
                <div class="flex flex-row py-1 box-border">
                    <label class="pr-2">"Remember me?"</label>
                    <input type="checkbox" name="remember"/>
                </div>
                <input
                    type="submit"
                    class="w-max py-1 px-2 rounded-md bg-primary-color hover:bg-hover-accent-color glow-hover"
                />
            </ActionForm>
            <ActionForm action=sign_up class="flex flex-col w-60">
                <h1 class="text-2xl">"Sign Up"</h1>
                <div class="flex flex-col py-1">
                    <label>"Display name:"</label>
                    <input
                        type="text"
                        placeholder="DisplayName"
                        maxlength="64"
                        name="display_name"
                        required
                    />
                </div>
                <div class="flex flex-col py-1">
                    <label>"Email:"</label>
                    <input type="email" placeholder="Email" maxlength="64" name="email" required/>
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
                    <progress
                        value=move || zxcvbn(password().as_str(), &[]).map_or(0u8, |e| e.score())

                        max=4
                        class=move || {
                            let password_strength = zxcvbn(password().as_str(), &[])
                                .map_or(0u8, |e| e.score());
                            let color = match password_strength {
                                0 | 1 => "bg-red-400",
                                2 => "bg-yellow-400",
                                3 | 4 => "bg-green-400",
                                _ => "bg-gray-400",
                            };
                            format!(
                                "w-full \
                        [&::-webkit-progress-bar]:rounded-lg \
                        [&::-webkit-progress-value]:rounded-lg \
                        [&::-webkit-progress-bar]:bg-slate-400 \
                        [&::-webkit-progress-value]:{color} \
                        [&::-moz-progress-bar]:{color}",
                            )
                        }
                    >
                    </progress>
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
                />
            </ActionForm>
        </div>
    }
}
