use leptos::{component, view, IntoView};
use leptos_router::A;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="h-12 mt-auto text-sm flex items-center justify-between p-5 bg-[#primary-color]">
            <div class="flex items-center justify-center mx-auto space-x-4">
                "Copyright 2023-2024 Andrew Peterson."
                <A
                    href="terms_of_service"
                    class="transition-colors duration-300 ease-in-out text-white py-1.5 px-1.5 bg-[#primary-color] rounded-md hover:bg-hover-background-color"
                >
                    "Terms of Service"
                </A>
                <A
                    href="credits"
                    class="transition-colors duration-300 ease-in-out text-white py-1.5 px-1.5 bg-[#primary-color] rounded-md hover:bg-hover-background-color"
                >
                    "Credits"
                </A>
                <A
                    href="support"
                    class="transition-colors duration-300 ease-in-out text-white py-1.5 px-1.5 bg-[#primary-color] rounded-md hover:bg-hover-background-color"
                >
                    "Support / FAQ"
                </A>
            </div>
        </footer>
    }
}

