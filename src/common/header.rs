use leptos::{component, spawn_local, view, IntoView, ReadSignal, WriteSignal};
use leptos_router::A;

use crate::server::{self};

#[component]
pub fn Header(logged_in: ReadSignal<bool>, set_logged_in: WriteSignal<bool>) -> impl IntoView {
    view! {
        <header class="bg-[#primary-color] border-b-4 border-black flex justify-between text-2xl items-center p-5">
            <img/>
            <nav class="nav">
                <A
                    href=""
                    class="transition-colors duration-300 ease-in-out text-white py-1.5 px-1.5 bg-secondary-color-color rounded-md hover:bg-hover-background-color"
                >
                    "Home"
                </A>
                <A
                    href="about"
                    class="transition-colors duration-300 ease-in-out text-white py-1.5 px-1.5 bg-[#primary-color] rounded-md hover:bg-hover-background-color"
                >
                    "About"
                </A>
                <A
                    href="community"
                    class="transition-colors duration-300 ease-in-out text-white py-1.5 px-1.5 bg-[#primary-color] rounded-md hover:bg-hover-background-color"
                >
                    "Community"
                </A>
                <A
                    href="support"
                    class="transition-colors duration-300 ease-in-out text-white py-1.5 px-1.5 bg-[#primary-color] rounded-md hover:bg-hover-background-color"
                >
                    "Help"
                </A>
                <A
                    href="checkout"
                    class="transition-colors duration-300 ease-in-out text-white py-1.5 px-1.5 bg-[#primary-color] rounded-md hover:bg-hover-background-color"
                >
                    "Buy Game"
                </A>
                {move || match logged_in() {
                    true => {
                        view! {
                            <A
                                href="download"
                                class="transition-colors duration-300 ease-in-out text-white py-1.5 px-1.5 bg-[#primary-color] rounded-md hover:bg-hover-background-color"
                            >
                                "Download"
                            </A>
                        }
                    }
                    false => view! { <div></div> }.into_view(),
                }}

            </nav>
            <div class="ml-auto justify-self-center">
                {move || match logged_in() {
                    true => {
                        view! {
                            <div>
                                <A
                                    href=""
                                    on:click=move |_| {
                                        set_logged_in(false);
                                        spawn_local(async {
                                            let _ = server::public::logout().await;
                                        })
                                    }

                                    class="transition-colors duration-300 ease-in-out text-white py-1.5 px-1.5 bg-[#primary-color] rounded-md hover:bg-hover-background-color"
                                >

                                    "Log out"
                                </A>
                            </div>
                        }
                    }
                    false => {
                        view! {
                            <div>
                                <A
                                    href="log_in"
                                    class="transition-colors duration-300 ease-in-out text-white py-1.5 px-1.5 bg-[#primary-color] rounded-md hover:bg-hover-background-color"
                                >
                                    "Log in"
                                </A>
                                |
                                <A
                                    href="log_in"
                                    class="transition-colors duration-300 ease-in-out text-white py-1.5 px-1.5 bg-[#primary-color] rounded-md hover:bg-hover-background-color"
                                >
                                    "Sign up"
                                </A>
                            </div>
                        }
                    }
                }}

            </div>
        </header>
    }
}

