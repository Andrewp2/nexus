use crate::{errors::NexusError, public::Logout, AccountState};
use leptos::{component, view, Action, IntoView, Memo, ServerFnError, SignalGet};
use leptos_router::A;

#[component]
pub fn LoginAndSignupLinks() -> impl IntoView {
    view! {
        <div>
            <A
                href="log_in"
                class=" text-t-color p-1.5 rounded-md hover:bg-hover-accent-color glow-hover"
            >
                "Log in"
            </A>
            |
            <A
                href="log_in"
                class=" text-t-color p-1.5 rounded-md hover:bg-hover-accent-color glow-hover"
            >
                "Sign up"
            </A>
        </div>
    }
}

#[component]
pub fn LogOutButton(
    logout_action: Action<Logout, Result<(), ServerFnError<NexusError>>>,
) -> impl IntoView {
    view! {
        <div>
            <A
                href=""
                on:click=move |_| {
                    logout_action.dispatch(Logout {});
                }

                class="text-t-color p-1.5 rounded-md hover:bg-hover-accent-color glow-hover"
            >

                "Log out"
            </A>
        </div>
    }
}

#[component]
pub fn Header(
    logout_action: Action<Logout, Result<(), ServerFnError<NexusError>>>,
    account_state: Memo<AccountState>,
) -> impl IntoView {
    view! {
        <header class="bg-white/5 border-b-4 border-white/10 flex justify-between text-2xl items-center p-5 ">
            <img/>
            <nav class="space-x-1">
                <A
                    href=""
                    class=" text-t-color p-1.5 rounded-md hover:bg-hover-accent-color glow-hover"
                >
                    "Home"
                </A>
                <A
                    href="about"
                    class=" text-t-color p-1.5 rounded-md hover:bg-hover-accent-color glow-hover"
                >
                    "About"
                </A>
                <A
                    href="https://discord.gg/EuqSvxDPRY"
                    class=" text-t-color p-1.5 rounded-md hover:bg-hover-accent-color glow-hover"
                >
                    "Discord"
                </A>
                <A
                    href="support"
                    class=" text-t-color p-1.5 rounded-md hover:bg-hover-accent-color glow-hover"
                >
                    "Help"
                </A>
                <A
                    href="checkout"
                    class=" text-t-color p-1.5 rounded-md bg-primary-color hover:bg-hover-accent-color glow-hover"
                >
                    "Buy Game"
                </A>
                {move || match account_state.get() {
                    AccountState::LoggedIn => {
                        view! {
                            <A
                                href="download"
                                class=" text-t-color p-1.5 rounded-md hover:bg-hover-accent-color glow-hover"
                            >
                                "Download"
                            </A>
                        }
                            .into_view()
                    }
                    AccountState::LoggedOut => view! {}.into_view(),
                }}

            </nav>
            <div class="ml-auto justify-self-center">
                {move || match account_state.get() {
                    AccountState::LoggedIn => view! { <LogOutButton logout_action=logout_action/> },
                    AccountState::LoggedOut => view! { <LoginAndSignupLinks/> },
                }}

            </div>
        </header>
    }
}
